```rust
/// The use case: it receives identity and coordinates domain behaviour. Application code does not check JWTs
/// or session cookies — it authorises an already-authenticated, provider-free principal and invokes domain
/// rules.
pub struct PledgingToOccasions<P> {
    pub persistence: P,
}

impl<P: PledgePersistence> PledgingToOccasions<P> {
    pub async fn pledge_to_occasion(&self, command: PledgeToOccasionCommand) -> Result<PledgeResult, sqlx::Error> {
        let Some(stored) = self.persistence.find_occasion_by_id(&command.occasion_id).await? else {
            return Ok(Err(PledgeRejection::NotFound));
        };
        // ... delegate to the domain function and save the outcome
    }
}
```
