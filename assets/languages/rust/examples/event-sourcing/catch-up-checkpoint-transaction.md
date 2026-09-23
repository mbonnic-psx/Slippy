```rust
/// The next event to apply is not immediately after the checkpoint — a position in between is missing, so
/// applying now would silently skip it.
#[derive(Debug, thiserror::Error)]
pub enum CatchUpError {
    #[error("projection gap: at position {at}, applied through {applied_through}; retry after the missing position")]
    Gap { at: u64, applied_through: u64 },
    #[error(transparent)]
    Corrupt(#[from] CorruptProjection),
    #[error(transparent)]
    Store(#[from] StoreError),
}

/// The transactional context one catch-up step needs: a lease, and the checkpoint and the view read and
/// written together. A real adapter backs it with its store's transaction; this stays at the port.
pub trait ProjectionTx {
    async fn acquire_exclusive_projection_lease(&mut self, projection: &str) -> Result<(), StoreError>;
    async fn load_checkpoint(&mut self, projection: &str) -> Result<u64, StoreError>;
    async fn save_checkpoint(&mut self, projection: &str, position: u64) -> Result<(), StoreError>;
    async fn load_balance_view(&mut self, stream_id: &StreamId) -> Result<BalanceView, StoreError>;
    async fn upsert_balance_view(&mut self, view: &BalanceView) -> Result<(), StoreError>;
}

/// Applies one envelope and advances the checkpoint atomically: an exclusive lease so no other worker races
/// this one, exact redelivery skipped, a gap refused loudly rather than skipped, and the view and the
/// checkpoint written in the same transaction. The caller commits on `Ok` and rolls back on `Err`.
pub async fn apply_checkpointed(
    tx: &mut impl ProjectionTx,
    projection: &str,
    envelope: &AccountProjectionEnvelope,
) -> Result<(), CatchUpError> {
    tx.acquire_exclusive_projection_lease(projection).await?;

    let applied_through = tx.load_checkpoint(projection).await?;
    if envelope.global_position <= applied_through {
        return Ok(()); // exact redelivery
    }
    if envelope.global_position != applied_through + 1 {
        return Err(CatchUpError::Gap { at: envelope.global_position, applied_through });
    }

    let current = tx.load_balance_view(&envelope.stream_id).await?;
    let next = apply(current, envelope)?;
    tx.upsert_balance_view(&next).await?;
    tx.save_checkpoint(projection, envelope.global_position).await?; // same transaction
    Ok(())
}
```
