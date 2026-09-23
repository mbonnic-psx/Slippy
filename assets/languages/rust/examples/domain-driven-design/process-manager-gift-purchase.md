```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OccasionId(pub String);

/// The lifecycle phases, each carrying only the fields valid in it.
#[derive(Debug, Clone, PartialEq)]
pub enum GiftPurchasePhase {
    AwaitingPayment { occasion_id: OccasionId },
    AwaitingShipment { payment_id: String },
    Complete { tracking_number: String },
    Failed { reason: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct GiftPurchaseProcess {
    pub phase: GiftPurchasePhase,
    pub processed_event_ids: Vec<String>,
}

/// The facts the process manager reacts to.
#[derive(Debug, Clone, PartialEq)]
pub enum GiftPurchaseEvent {
    PaymentSucceeded { id: String, payment_id: String },
    PaymentFailed { id: String },
    GiftShipped { id: String, tracking_number: String },
}

impl GiftPurchaseEvent {
    fn id(&self) -> &str {
        match self {
            Self::PaymentSucceeded { id, .. } | Self::PaymentFailed { id } | Self::GiftShipped { id, .. } => id,
        }
    }
}

/// The follow-up commands the process manager emits.
#[derive(Debug, Clone, PartialEq)]
pub enum GiftPurchaseCommand {
    ShipGift { payment_id: String, idempotency_key: String },
    ReleaseBudgetHold { occasion_id: OccasionId, idempotency_key: String },
}

/// A real state change, or a no-op said out loud — duplicates and out-of-order events are refused, never
/// silently swallowed.
#[derive(Debug, PartialEq)]
pub enum ProcessReaction {
    Applied { new_state: GiftPurchaseProcess, commands: Vec<GiftPurchaseCommand> },
    Duplicate,
    OutOfOrder,
}

/// Applies each event once, and only in the phase that can consume it.
pub fn advance_gift_purchase(state: &GiftPurchaseProcess, event: &GiftPurchaseEvent) -> ProcessReaction {
    if state.processed_event_ids.iter().any(|id| id == event.id()) {
        return ProcessReaction::Duplicate;
    }
    let mut processed_event_ids = state.processed_event_ids.clone();
    processed_event_ids.push(event.id().to_owned());

    let (phase, commands) = match (&state.phase, event) {
        (GiftPurchasePhase::AwaitingPayment { .. }, GiftPurchaseEvent::PaymentSucceeded { id, payment_id }) => (
            GiftPurchasePhase::AwaitingShipment { payment_id: payment_id.clone() },
            vec![GiftPurchaseCommand::ShipGift { payment_id: payment_id.clone(), idempotency_key: id.clone() }],
        ),
        (GiftPurchasePhase::AwaitingPayment { occasion_id }, GiftPurchaseEvent::PaymentFailed { id }) => (
            GiftPurchasePhase::Failed { reason: "payment-declined".into() },
            vec![GiftPurchaseCommand::ReleaseBudgetHold { occasion_id: occasion_id.clone(), idempotency_key: id.clone() }],
        ),
        (GiftPurchasePhase::AwaitingShipment { .. }, GiftPurchaseEvent::GiftShipped { tracking_number, .. }) => {
            (GiftPurchasePhase::Complete { tracking_number: tracking_number.clone() }, Vec::new())
        }
        _ => return ProcessReaction::OutOfOrder,
    };
    ProcessReaction::Applied { new_state: GiftPurchaseProcess { phase, processed_event_ids }, commands }
}
```
