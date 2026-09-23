```rust
/// The driving port — exposed by the application, called by driving adapters.
pub trait ForPlacingOrders {
    async fn place_order(&self, command: PlaceOrderCommand) -> Result<PlaceOrderResult, PlaceError>;
}

/// A driven port — application-owned because the use case consumes it.
pub trait UserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, RepositoryError>;
    async fn save(&self, user: &User) -> Result<(), RepositoryError>;
}

/// The two expected results of preparing a payment. A business outcome, not an error — an unexpected gateway
/// failure is still the `Err` of the `Result` around it.
#[derive(Debug, PartialEq)]
pub enum PreparePaymentResult {
    Prepared { payment_id: PaymentId },
    Failed { reason: PaymentFailure },
}

#[derive(Debug, PartialEq)]
pub enum PaymentOutcome {
    Paid { charge_id: ChargeId },
    Declined { reason: PaymentFailure },
    Pending,
}

/// A driven port — application-owned because the use case consumes it. It models a two-phase
/// prepare/complete payment conversation.
pub trait PaymentGateway {
    async fn prepare_payment(&self, amount: Money, reference: &OrderId, idempotency_key: &OrderId)
        -> Result<PreparePaymentResult, GatewayError>;
    async fn complete_payment(&self, payment_id: &PaymentId, payment_info: &PaymentInfo)
        -> Result<PaymentOutcome, GatewayError>;
}

/// A driven port for publishing events outbound to a message broker.
pub trait OrderEventPublisher {
    async fn publish(&self, event: &OrderEvent) -> Result<(), PublishError>;
}
```
