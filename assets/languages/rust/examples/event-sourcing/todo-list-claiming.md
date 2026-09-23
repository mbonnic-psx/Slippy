```rust
// The todo list is a table, folded from earlier slices' events by the checkpointed runner in
// catch-up-checkpoint-transaction and read here. It is still a projection — PlaceClaimed inserts the row,
// SeatAllocated removes it — so it rebuilds from position zero like any other. What it is not is a fold
// over what the current request happened to write: a standalone run has no request to ask.

/// One row of the todo list: a claim with no seat yet.
#[derive(Debug, Clone)]
pub struct OwedSeat {
    pub registration_id: String,
    pub claimed_at: String,
}

/// Hands out work no other runner is holding, for the life of the caller's transaction.
pub trait SeatsOwed {
    async fn claim_batch(&self, limit: u32) -> Result<Vec<OwedSeat>, StoreError>;
}
```

```sql
-- Claiming is what lets a second runner help rather than queue: SKIP LOCKED takes the next unlocked row
-- instead of blocking behind the first. Note what it is *not* doing — it is not what makes the work happen
-- once. Between issuing AllocateSeat and the projection applying SeatAllocated the row is still here, so a
-- run that overlaps its predecessor can re-issue the command; the Decider refuses it, because
-- folds: [SeatAllocated, SeatReleased] is its own stream's history and a seat already allocated is a
-- rejection. Correctness comes from that invariant. Claiming only stops two runners paying for the same
-- work twice.
SELECT registration_id, claimed_at
  FROM seats_owed
 ORDER BY claimed_at
 LIMIT $1
   FOR UPDATE SKIP LOCKED
```

```rust
/// Claims work, decides, and issues the command — through the same use case a person would have driven.
///
/// Invoked in-request for latency and runnable standalone: on a timer, at startup, after a crash. The
/// standalone run is the point of the pattern. It discovers work it did not create, and it recovers what a
/// request that died halfway abandoned — neither of which a per-request fold can do at all.
pub async fn allocate_owed_seats(
    owed: &impl SeatsOwed,
    allocate: &impl AllocateSeat,
    batch: u32,
) -> Result<usize, AllocateError> {
    let claims = owed.claim_batch(batch).await.map_err(AllocateError::Claim)?;
    let mut issued = 0;
    for claim in claims {
        let outcome = allocate
            .allocate(&claim.registration_id)
            .await
            .map_err(|error| AllocateError::Allocate { registration: claim.registration_id.clone(), error })?;
        // A refusal appends nothing, so nothing removes the row and the next run sees it again. That is the
        // compensation gap: decide whether the refusal becomes an event or the todo list detects the age of
        // the claim, and write the decision down — a processor that silently retries forever is neither.
        if outcome.allocated {
            issued += 1;
        }
    }
    Ok(issued)
}
```
