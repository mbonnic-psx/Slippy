```rust
/// A driven port, application-owned because the use case consumes it. Probe methods take domain types only,
/// return nothing a caller branches on, and never steer control flow. No log levels, no metric names, no
/// framework types.
pub trait PledgeInstrumentation {
    fn pledge_rejected(&self, reason: RejectionReason, occasion_id: &OccasionId);
    fn pledge_accepted(&self, amount: Money, occasion_id: &OccasionId);
}

/// Announces domain facts through the probe; the adapter decides what they are worth.
pub struct PledgingToOccasions<P, I> {
    pub persistence: P,
    pub instrumentation: I,
}

impl<P: PledgePersistence, I: PledgeInstrumentation> PledgingToOccasions<P, I> {
    pub async fn pledge_to_occasion(&self, command: PledgeToOccasionCommand) -> Result<PledgeResult, PersistenceError> {
        // ... calls self.instrumentation.pledge_rejected / pledge_accepted as outcomes occur
    }
}

/// The adapter: it decides severity, field names and span attributes — swappable without touching a use case.
pub struct TracingPledgeInstrumentation;

impl PledgeInstrumentation for TracingPledgeInstrumentation {
    fn pledge_rejected(&self, reason: RejectionReason, occasion_id: &OccasionId) {
        tracing::warn!(?reason, occasion_id = %occasion_id, "pledge rejected");
    }

    fn pledge_accepted(&self, amount: Money, occasion_id: &OccasionId) {
        tracing::info!(minor_units = amount.minor_units, currency = ?amount.currency, occasion_id = %occasion_id, "pledge accepted");
    }
}
```
