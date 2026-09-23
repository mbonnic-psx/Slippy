```rust
// ❌ Names the pattern or the type mechanism
pub trait PaymentPort {
    async fn charge(&self, amount: Money, info: &PaymentInfo) -> Result<ChargeResult, GatewayError>;
}

pub struct PaymentGatewayImpl {
    api_key: String,
}

impl PaymentPort for PaymentGatewayImpl {
    async fn charge(&self, amount: Money, info: &PaymentInfo) -> Result<ChargeResult, GatewayError> {
        // ...
    }
}

pub async fn place_order_use_case(order: NewOrder) -> Result<OrderResult, PlaceError> {
    // ...
}

// ✅ Names the role and the concrete adapter
pub trait PaymentGateway {
    async fn charge(&self, amount: Money, info: &PaymentInfo) -> Result<ChargeResult, GatewayError>;
}

pub struct StripePaymentGateway {
    api_key: String,
}

impl PaymentGateway for StripePaymentGateway {
    async fn charge(&self, amount: Money, info: &PaymentInfo) -> Result<ChargeResult, GatewayError> {
        // ...
    }
}

pub async fn place_order(order: NewOrder) -> Result<OrderResult, PlaceError> {
    // ...
}
```
