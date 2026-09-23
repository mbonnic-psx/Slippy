```rust
// gifting/src/hexagon/application/pledge_to_occasion.rs — use case

/// Implements `ForPledgingToOccasions`. Its driven port is a plain field, and the outer `Result` is reserved
/// for unexpected infrastructure failure — never for an expected outcome such as funding-closed or
/// concurrent-change, which is the inner one.
pub struct PledgingToOccasions<P> {
    pub persistence: P,
}

impl<P: PledgePersistence> ForPledgingToOccasions for PledgingToOccasions<P> {
    async fn pledge_to_occasion(&self, command: PledgeToOccasionCommand) -> Result<PledgeResult, PersistenceError> {
        let Some(stored) = self.persistence.find_occasion_by_id(&command.occasion_id).await? else {
            return Ok(Err(PledgeRejection::NotFound));
        };

        let pledged = match record_pledge(stored.value, command.pledge_id, command.principal.contributor_id, command.amount) {
            Ok(pledged) => pledged,
            Err(reason) => return Ok(Err(reason.into())),
        };

        match self.persistence.save_with_outbox(&pledged.occasion, &pledged.events, stored.version).await? {
            SaveOutcome::Saved => Ok(Ok(pledged)),
            SaveOutcome::Conflict => Ok(Err(PledgeRejection::ConcurrentChange)),
        }
    }
}

/// Records a `PledgeRecorded` event into the projection. The outbox worker retries delivery, so this has to
/// stay safe when the same event arrives twice — a guarantee that lives in the `PledgeProjection`
/// implementation, not here.
pub async fn handle_pledge_recorded(projection: &impl PledgeProjection, event: &PledgeRecorded) -> Result<(), PersistenceError> {
    projection.record_from(event).await
}
```
