# Projections and Read Models

> Examples are TypeScript-as-pseudocode until idiomatic Python versions land.

The event log is the source of truth, but it only answers "give me this stream's events." Every *query* — dashboards, lists, detail pages, search — is served by a **read model** built by projecting event envelopes. A projection is another left fold: `envelopes.reduce(apply, emptyBalanceView)`. This is the read side of CQRS, taken to its full form.

## A Projection Is a Fold

An `apply` function has the same shape as `evolve`, but its target is a query-optimised view rather than the aggregate's decision state:

```typescript
type BalanceView =
  | { readonly status: 'unopened' }
  | {
      readonly status: 'open';
      readonly accountId: StreamId;
      readonly balanceMinorUnits: number;
      readonly currency: Currency;
    };

type AccountProjectionEnvelope = {
  readonly streamId: StreamId;
  readonly globalPosition: bigint;
  readonly data: AccountEvent;
};

const emptyBalanceView: BalanceView = { status: 'unopened' };

const corruptBalanceProjection = (view: BalanceView, event: AccountEvent): never => {
  throw new Error(`Corrupt balance projection: ${event.type} cannot follow ${view.status}`);
};

const isPositiveMinorUnits = (minorUnits: number): boolean =>
  Number.isSafeInteger(minorUnits) && minorUnits > 0;

const addMinorUnits = (left: number, right: number): number => {
  if (!Number.isSafeInteger(left) || !Number.isSafeInteger(right)) {
    throw new Error('Corrupt account history: unsafe minor-unit operand');
  }
  const result = left + right;
  if (!Number.isSafeInteger(result) || result < 0) {
    throw new Error('Corrupt account history: invalid projected balance');
  }
  return result;
};

const apply = (view: BalanceView, envelope: AccountProjectionEnvelope): BalanceView => {
  const event = envelope.data;
  switch (event.type) {
    case 'AccountOpened':
      if (view.status !== 'unopened') return corruptBalanceProjection(view, event);
      return {
        status: 'open',
        accountId: envelope.streamId,
        currency: event.currency,
        balanceMinorUnits: 0,
      };
    case 'MoneyDeposited':
      if (
        view.status !== 'open' ||
        view.accountId !== envelope.streamId ||
        !isPositiveMinorUnits(event.amount.minorUnits) ||
        event.amount.currency !== view.currency
      ) return corruptBalanceProjection(view, event);
      return {
        ...view,
        balanceMinorUnits: addMinorUnits(view.balanceMinorUnits, event.amount.minorUnits),
      };
    case 'MoneyWithdrawn':
      if (
        view.status !== 'open' ||
        view.accountId !== envelope.streamId ||
        !isPositiveMinorUnits(event.amount.minorUnits) ||
        event.amount.currency !== view.currency ||
        event.amount.minorUnits > view.balanceMinorUnits
      ) return corruptBalanceProjection(view, event);
      return {
        ...view,
        balanceMinorUnits: addMinorUnits(view.balanceMinorUnits, -event.amount.minorUnits),
      };
    default: { const _: never = event; return _; }
  }
};
```

`AccountOpened` is the creation path: it derives `accountId` from the stored envelope. The consumer skips an exact redelivery by event ID or checkpoint before calling `apply`; an admitted later opening, a money event before opening, or an envelope for another stream is impossible known history and must surface corruption instead of manufacturing or resetting a row. Unknown stored types are handled by tolerant deserialization and upcasting before this exhaustive typed fold.

A projection may fold **across many streams** into one view (a "customers with overdrafts" table), which is exactly what aggregate boundaries forbid on the write side and exactly why the read side is separate. Keep **derivation logic in the events themselves** (the rate that applied, the amount charged) so projections stay simple, deterministic, and replayable — do not make a projection recompute business values.

## Inline vs Async

There are three lifecycles, and the choice is a consistency/latency trade-off (Marten's taxonomy):

| Lifecycle | When it runs | Consistency | Cost |
|-----------|-------------|-------------|------|
| **Inline (synchronous)** | In the same transaction as the append | Strong — the view always matches the log | Adds latency to every write; couples write throughput to projection work |
| **Live** | On demand, per query, folded in memory (not persisted) | Strong | Recomputed every read, forever; viable only while the fold stays inside a written-down ceiling |
| **Async** | A background subscription, in its own transaction | **Eventual** — the view lags the write | Scales; isolates failures; adds a lag to manage |

### What each one looks like

`live` folds per query and stores nothing, so the only code it needs is the fold and the ceiling it holds
inside — the ceiling asserted, because a per-query fold gets slower by a millisecond a week and never fails
until a page times out:

```typescript
// `materialisation: live` — the view is folded per query and nothing is stored. No table, no checkpoint,
// no subscription, no rebuild path, and strongly consistent by construction: it reads the log the command
// just wrote. That is why it is the answer taken by default, and why the slice has to declare the ceiling
// it holds inside. Here the ceiling is asserted rather than assumed.

/**
 * `liveBudget.events` for this view, and the argument for it belongs beside the number: one account
 * stream, which ends at closure, so the ceiling is the busiest account's lifetime and not a guess about
 * traffic. Raise it deliberately, or materialise the view — do not raise it because a test went red.
 */
const MAX_EVENTS_FOLDED = 40;

export class ViewOutgrewItsBudget extends Error {}

export async function readBalanceView(events: EventStore, accountId: StreamId): Promise<BalanceView> {
  const history = await events.read(accountId);
  if (history.length > MAX_EVENTS_FOLDED) {
    // Failing is the point. A per-query fold does not degrade visibly — it gets slower by a millisecond a
    // week until a page times out, and by then the fix is a table, a checkpoint, a backfill and every
    // caller. This turns that into one red test on the day the model changed.
    throw new ViewOutgrewItsBudget(
      `balance view folded ${String(history.length)} events for ${accountId}, over its budget of ` +
        `${String(MAX_EVENTS_FOLDED)}: close the stream at a business boundary, or materialise the view`,
    );
  }
  return history.reduce<BalanceView>(
    (view, committed) =>
      applyToBalanceView(view, {
        streamId: committed.streamId,
        globalPosition: committed.globalPosition,
        data: toDomainEvent(committed),
      }),
    emptyBalanceView,
  );
}
```

```typescript
// tests/projection/balance-view-budget.test.ts — the test that makes the ceiling a fact
it('fails on a stream past the budget rather than getting slower', async () => {
  const account = await openAccountWithDeposits(events, MAX_EVENTS_FOLDED);

  await deposit(events, account, 1);

  await expect(readBalanceView(events, account)).rejects.toThrow(ViewOutgrewItsBudget);
});
```

`inline` writes the view in the append's own transaction, through the store's unit-of-work seam — a block
inside which `append` does not commit, so the view row and the events it derives from arrive together or
not at all, and a failed view write takes the append down with it. On the JVM that seam *is* the
framework's: the Postgres store takes the transaction manager as a port, so a `@Transactional` service
method puts the append and the project's own repository in one transaction with nothing wired between
them:

```typescript
// `materialisation: inline` — the view is written in the same transaction as the append, so it never lags
// the write and read-your-writes costs nothing on this slice. Two things to know before choosing it.
//
// The seam this needs is `EventStore.inUnitOfWork`, which the store you were given already has: a callback
// inside which `append` does not commit, so a view write beside it commits with the events it derives from,
// or neither happens. Any adapter built from the same store's session is inside that transaction too —
// which is why the view's own store below is constructed from it rather than from a pool of its own.
//
// The view must be scoped to the stream being appended. A view folding several streams cannot be kept
// atomic with one append: the row it writes is contended by every other stream's appends, which is a hot
// row and a cross-stream transaction wearing a projection's clothes. That view is `async`.

/**
 * This project's own view table, on the event store's session.
 *
 * Built from the store — `createBalanceViewStore(store)` — for the same reason the checkpoint store is: a
 * write on a second connection is a second transaction, and then "inline" is a word rather than a
 * guarantee.
 */
export interface BalanceViewStore {
  load(accountId: StreamId): Promise<BalanceView | undefined>;
  upsert(view: BalanceView): Promise<void>;
}

export async function depositMoney(
  store: EventStore,
  views: BalanceViewStore,
  command: DepositMoney,
): Promise<DepositOutcome> {
  return store.inUnitOfWork(async () => {
    const history = await store.read(command.accountId);
    const decision = accountDecider.decide(command, rehydrate(accountDecider, history));
    if (decision.outcome === 'rejected') return decision; // nothing appended, nothing projected

    const result = await store.append(command.accountId, currentVersion(history), decision.events);
    // Contention, not failure: the caller re-reads and re-decides. Nothing is half-written, which is the
    // one thing inline gives you for free.
    if (result.outcome === 'version-conflict') return result;

    // The same `applyToBalanceView` the `live` and `async` versions use — the lifecycle decides what
    // maintains the view, never how it is computed. Only the new events are applied: refolding the stream
    // here would put the whole history on the write path, which is the cost inline exists to avoid.
    let view = (await views.load(command.accountId)) ?? emptyBalanceView;
    for (const committed of (await store.read(command.accountId)).slice(history.length)) {
      view = applyToBalanceView(view, {
        streamId: committed.streamId,
        globalPosition: committed.globalPosition,
        data: toDomainEvent(committed),
      });
    }
    await views.upsert(view);
    return result;
  });
}
```

An inline view is still a derivation, so it still needs the rebuild path: when the fold changes or turns out
to be wrong, the fix is to reset the view and replay `readAll(0)` through the same `apply`. Inline removes
the checkpoint and the subscription, not the obligation to be rebuildable.

`async` is the catch-up subscription below — the fold, a checkpoint advanced in the projection's own
transaction, ordered ownership, and a rebuild path. In a project generated by this factory it is *not* the
most code by a distance any more, because all of that except the fold is shipped: `CheckpointStore` with an
adapter per store, `catchUp`, `catchUpEach` and `rebuild` beside it, and one contract suite run against
every adapter. **What runs it is the framework's own mechanism, not a worker loop this project wrote** — a
`@Scheduled` bean under Quarkus and Spring Boot, a Fastify plugin whose `onClose` stops the timer, a FastAPI
lifespan started and stopped with the app, a blocking `Run(ctx)` under the context `cmd/serve` cancels. Each
of them is inert until the project has a store and at least one projection, so a project with only `live`
and `inline` views has nothing running. What the choice costs is still a real difference — a lag to cap, a
process to watch — and it is still a choice.

Default to **async** for anything that must scale or fan out, and reserve **inline** for the few views where a user must never see stale data immediately after their own write. Do not make everything inline "to be safe" — that throws away the isolation that makes event sourcing operable.

**Which one is not a decision to leave implicit, because the implicit answer is always `live`.** A store
that can `read` a stream and replay from zero is all a per-query fold needs, and for a long time in this
factory's own output it was the only lifecycle with anything already built — an implementer who was never
asked took it without noticing there was a question. The read side ships now, so the gap has closed; the
*question* is what mattered, and it is still asked. In a project generated by this factory the
answer is a field on the slice, `materialisation: live | inline | async`, and `make check-model` requires it
before a state-view or automation slice can be planned (`delivery/docs/event-model/README.md`, *Where the read model
lives*). Elsewhere, write it in the plan next to the store.

**A `live` fold carries the ceiling it holds inside.** *"Only viable for short streams"* is a condition, and
a condition nobody wrote down is an assumption nobody can check: name the maximum number of events one query
may fold and why that number holds — the stream has an end event, or a closing-the-books summary starts a
fresh one — and assert it in the projection's own test, so a stream that outgrows the ceiling fails a test
instead of quietly getting slower in production. The model's `liveBudget` is exactly those two values.

## Catch-up Subscriptions and Checkpoints

An async projection is driven by a **catch-up subscription**: it reads events in global order from a stored position, applies each, and advances a **checkpoint** (the last position processed). On startup it resumes from the checkpoint; with none, it starts from the beginning.

Store the checkpoint in the same transaction as the projection update, but do
not mistake that necessary step for complete ordering. One exclusive,
sequential catch-up owner (for example, a transaction-scoped lease) must own
each projection, or an equivalent locked/gap-aware protocol must prevent
position N+1 from committing before N. Delivery remains at-least-once; the
transaction prevents duplicate committed projection effects only while that
ordered-ownership contract holds.

**The gap that protocol is about is not hypothetical, and on Postgres it is the default.** A global
position is handed out when a row is *inserted*; it becomes visible when that row's transaction *commits*.
Two appends overlapping take 5 and 6, and 6 can commit first — so a subscription that reads 6, applies it
and records "next is 7" has skipped 5 for good: 5 arrives behind a checkpoint that has already passed it.
The log is correct, the view is missing a row, nothing complains, and a rebuild is the only cure. Every
scalar-checkpoint subscription needs an answer to this, whatever store it is on.

**In a project generated by this factory, that is what `CheckpointStore` and the runner beside it are.**
`positionOf` and `record` are the checkpoint — `record` deliberately does not commit, because the caller's
unit of work is what commits it with the rows it accounts for — and `claim`/`release` are the exclusive
owner, as a *lease with an expiry* rather than a lock, so a worker that dies holding it does not stop the
projection for good. `catchUp` is one pass of the loop below and `rebuild` is the same pass from zero with
the view emptied first, holding the lease across both. Its contract suite runs against memory, SQLite and
Postgres, and the case worth reading is the one that fails a checkpoint recorded inside a unit of work that
threw. The snippet below is what all of that does, in the small.

The gap above is answered in the adapter rather than in the runner, so a projection never sees it: on
Postgres an append holds the log's advisory lock in **shared** mode from its first insert until it commits,
and `readAll` takes that lock **exclusively** for one `MAX(global_position)` query. Holding it exclusively
means no append is between its insert and its commit, so every position at or below that maximum is
settled — committed and visible, or aborted and gone — and the replay stops there. Appends never block each
other; a replay waits for the appends in flight, which is a pause and not a skip. SQLite and the in-memory
store need none of it: one writer at a time cannot commit out of order.

What *calls* it is the framework, in the shape that framework already has: `@Scheduled` with
`ConcurrentExecution.SKIP` (Quarkus) or a `fixedDelay` (Spring Boot), a Fastify plugin, a FastAPI
lifespan, a blocking `Run(ctx)` in Go. Two details in all four are worth copying into anything you
write yourself: **one pass at a time** — the pause measured from the end of a pass, not on a fixed
schedule, so a slow pass cannot have another pile up behind it — and **every projection attempted**,
with the failures reported together at the end, so one broken fold does not starve every projection
after it in the list.

```typescript
// one event, applied and checkpointed atomically
await db.transaction(async (tx) => {
  await acquireExclusiveProjectionLease(tx, projectionName);
  const appliedThrough = await loadCheckpoint(tx, projectionName) ?? 0n;
  if (envelope.globalPosition <= appliedThrough) return; // exact redelivery
  if (envelope.globalPosition !== appliedThrough + 1n) {
    throw new Error('Projection gap: retry after the missing position');
  }
  const current = await loadBalanceView(tx, envelope.streamId) ?? emptyBalanceView;
  await upsertBalanceView(tx, apply(current, envelope));
  await saveCheckpoint(tx, projectionName, envelope.globalPosition); // same transaction
});
```

Exercise two concurrent workers, a crash before commit, and an out-of-order
position in the projection tests. A competing consumer without ordered
ownership must not share this scalar checkpoint.

Use **catch-up** (ordered, position-tracked) subscriptions whenever processing order matters. Reserve **persistent** (competing-consumer) subscriptions for order-insensitive, best-effort work — they scale out but do not preserve order.

## A Todo List Is Claimed as Well as Folded

A projection that feeds a **processor** — an automation's todo list — is read by something that then *acts*,
so two runners must not act on one row. Claim the batch with `FOR UPDATE SKIP LOCKED` (or the store's
equivalent), which lets a second runner take the next free row instead of queueing behind the first:

```typescript
// The todo list is a table, folded from earlier slices' events by the checkpointed runner in
// `catch-up-checkpoint-transaction` and read here. It is still a projection — `PlaceClaimed` inserts the
// row, `SeatAllocated` removes it — so it rebuilds from position zero like any other. What it is not is a
// fold over what the current request happened to write: a standalone run has no request to ask.

/** One row of the todo list: a claim with no seat yet. */
export type OwedSeat = Readonly<{ registrationId: string; claimedAt: string }>;

export interface SeatsOwed {
  /** Up to `limit` rows no other runner is holding, for the life of this transaction. */
  claimBatch(limit: number): Promise<readonly OwedSeat[]>;
}
```

```sql
-- Claiming is what lets a second runner help rather than queue: SKIP LOCKED takes the next unlocked row
-- instead of blocking behind the first. Note what it is *not* doing — it is not what makes the work happen
-- once. Between issuing AllocateSeat and the projection applying SeatAllocated the row is still here, so a
-- run that overlaps its predecessor can re-issue the command; the Decider refuses it, because
-- `folds: [SeatAllocated, SeatReleased]` is its own stream's history and a seat already allocated is a
-- rejection. Correctness comes from that invariant. Claiming only stops two runners paying for the same
-- work twice.
SELECT registration_id, claimed_at
  FROM seats_owed
 ORDER BY claimed_at
 LIMIT $1
   FOR UPDATE SKIP LOCKED
```

```typescript
/**
 * Claim work, decide, issue the command — through the same use case a person would have driven.
 *
 * Invoked in-request for latency *and* runnable standalone: on a timer, at startup, after a crash. The
 * standalone run is the point of the pattern. It discovers work it did not create, and it recovers what a
 * request that died halfway abandoned — neither of which a per-request fold can do at all, because it only
 * knows the streams the request in front of it wrote.
 */
export async function allocateOwedSeats(
  owed: SeatsOwed,
  allocate: (command: AllocateSeat) => Promise<AllocationOutcome>,
  batch = 100,
): Promise<number> {
  let issued = 0;
  for (const claim of await owed.claimBatch(batch)) {
    const outcome = await allocate({ registrationId: claim.registrationId });
    // A refusal appends nothing, so nothing removes the row and the next run sees it again. That is the
    // compensation gap: decide whether the refusal becomes an event or the todo list detects the age of the
    // claim, and write the decision down — a processor that silently retries forever is neither.
    if (outcome.outcome === 'allocated') issued += 1;
  }
  return issued;
}
```

Two things about that snippet are the whole pattern, and both are easy to get backwards. **The rows are
derived, not marked done by the processor** — the triggering event inserts the row and the resulting event
removes it, so the todo list rebuilds from position zero like any other projection; a `done_at` the
processor writes itself is state nothing can rebuild. And **claiming is an efficiency measure, not what
makes the work happen once**: until the projection catches up the row is still there, so an overlapping run
re-issues the command, and what refuses it is the Decider's own invariant folded from its own stream. Build
it the other way round — correctness resting on the lock, the row marked done by hand — and both properties
are gone at the first crash.

## Idempotency Is Mandatory

Because at-least-once delivery permits duplicate attempts, a projection **may**
see the same event twice (for example, after a crash or replay overlap).
Applying it twice must not double-count. Make projections idempotent by either:

- keying the write on the **event id** or **global position** — values unique across the whole store — so a repeat is a no-op (e.g. `INSERT … ON CONFLICT (event_id) DO NOTHING`). A bare per-stream `version` guard (`WHERE last_version < :version`) only works when the read-model row is scoped to a single stream, because stream versions repeat across streams; or
- using the checkpoint so already-processed positions are skipped.

A projection that adds to a running total without guarding against redelivery is a latent data-corruption bug.

## Eventual Consistency and Read-Your-Writes

This is the headline operational surprise, and it is fundamental to the pattern, not a defect. With async projections the write model is strongly consistent but a read model **lags** — so a user who just issued a command may not immediately see it reflected. The canonical failure is POST-redirect-GET: publish a post, redirect to view it, and the read model has not caught up, so the user hits a 404.

Four mitigations (Ben Smith), in rough order of preference:

1. **Return the result from the command.** The command handler already knows the new state (it just folded the new events forward) — return it, or construct the view row from the command, and render *that* immediately. Simple and usually enough.
2. **Strong-consistency read on demand.** For specific views, read the aggregate's own stream (or an inline projection) so that path is strongly consistent, while everything else stays async.
3. **Subscribe / push.** Notify the client (WebSocket, SSE) once the projection has processed up to the relevant position.
4. **Poll** the read model until it reflects the expected version.

Whichever you choose, **design the UI for lag** — optimistic rendering, "saved, updating…" affordances — rather than pretending the read model is instantaneous. Monitor and cap projection lag as a first-class operational metric.

## Projections Are Disposable — Rebuild Them

Because state is `fold(events)`, any read model can be dropped and rebuilt from event zero. This is what makes two things trivial that are painful in a state-stored system:

- **Adding a new read model retroactively** — write the projection, replay history, and it is populated as if it had always existed.
- **Fixing a buggy projection** — correct the `apply` function, reset the checkpoint and the view, and replay. The events were always right; only the derivation was wrong.

Operational notes: for large stores, rebuild into **new tables alongside the live ones** and switch traffic once caught up (a blue/green / projection-versioning rebuild) so there is no downtime and no half-populated view being served. Never "fix" a read model by hand-editing rows — fix the projection and replay, or the next rebuild silently reverts your edit.

## Where This Sits Relative to CQRS-lite

The hexagonal skill's `cqrs-lite.md` is the same write/read split without event sourcing: writes through repositories, reads through query functions that JOIN. Event sourcing is the full version of that split — writes through the decider + event store, reads through projected read models folded from events. If a context does not need event history, stop at CQRS-lite; do not add projections and subscriptions for their own sake.
