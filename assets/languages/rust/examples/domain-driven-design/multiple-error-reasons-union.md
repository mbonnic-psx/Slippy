```rust
/// Every way creating an order can fail, as one closed set of specific reasons — not a generic error type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderRejection {
    EmptyCart,
    ItemOutOfStock,
    PaymentDeclined,
    AddressInvalid,
    DailyLimitExceeded,
}

pub type CreateOrderResult = Result<Order, OrderRejection>;
```
