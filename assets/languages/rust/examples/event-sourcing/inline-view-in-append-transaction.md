```rust
// materialisation: inline — the view is written in the same transaction as the append, so it never lags the
// write and read-your-writes costs nothing on this slice. Two things to know before choosing it.
//
// The seam this needs is the store's unit of work, which the store you were given already has: it hands the
// work a transaction, and inside it `append` does not commit — so a view write through that same transaction
// commits with the events it derives from, or neither happens. Pass the transaction to everything that must
// be part of it; a call given the pool instead is not.
//
// The view must be scoped to the stream being appended. A view folding several streams cannot be kept
// atomic with one append: its row is contended by every other stream's appends, which is a hot row and a
// cross-stream transaction wearing a projection's clothes. That view is async.

/// This project's own view table, built from the event store for the reason the checkpoint store is: a write
/// on a second connection is a second transaction, and then "inline" is a word rather than a guarantee.
pub trait BalanceViews {
    async fn load(&self, tx: &mut Tx, account_id: &StreamId) -> Result<Option<BalanceView>, StoreError>;
    async fn upsert(&self, tx: &mut Tx, view: &BalanceView) -> Result<(), StoreError>;
}

pub async fn deposit_money(
    store: &impl TransactionalEventStore,
    views: &impl BalanceViews,
    command: &DepositMoney,
) -> Result<Outcome, HandleError> {
    store
        .in_unit_of_work(async |tx: &mut Tx| {
            let history = store.read_in(tx, &command.account_id).await?;
            let state = rehydrate(&history)?;
            let new_events = match decide_deposit(command, state) {
                Ok(events) => events,
                // Nothing appended, nothing projected; returning without writing commits nothing.
                Err(rejection) => return Ok(Outcome::Rejected(rejection)),
            };
            let version = match store.append_in(tx, &command.account_id, current_version(&history), &new_events).await? {
                AppendResult::Appended { version } => version,
                // Contention, not failure: the caller re-reads and re-decides. Nothing is half-written, which
                // is the one thing inline gives you for free.
                AppendResult::VersionConflict { actual } => return Ok(Outcome::Conflict { actual }),
            };
            // The same `apply_to_balance_view` the live and async versions use — the lifecycle decides what
            // maintains the view, never how it is computed. Only the new events are applied: refolding the
            // stream here would put the whole history on the write path, which inline exists to avoid.
            let mut view = views.load(tx, &command.account_id).await?.unwrap_or(EMPTY_BALANCE_VIEW);
            let appended = store.read_in(tx, &command.account_id).await?;
            for committed in &appended[history.len()..] {
                let envelope = AccountProjectionEnvelope {
                    stream_id: committed.stream_id.clone(),
                    global_position: committed.global_position,
                    data: to_domain_event(committed)?,
                };
                view = apply_to_balance_view(view, &envelope)?;
            }
            views.upsert(tx, &view).await?;
            Ok(Outcome::Appended { version })
        })
        .await
}
```

An inline view is still a derivation, so it still needs the rebuild path: when the fold changes or turns out
to be wrong, the fix is to reset the view and replay `read_all(0)` through the same apply. Inline removes
the checkpoint and the subscription, not the obligation to be rebuildable.
