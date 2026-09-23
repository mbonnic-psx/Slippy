```rust
use serde::Deserialize;

/// The shape `OrderPlaced` was first persisted in: a flat total and currency.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OrderPlacedV1 {
    pub order_id: String,
    pub total_minor_units: i64,
    pub currency: Currency,
}

/// The current persisted shape: the flat total restructured into a nested amount.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OrderPlacedV2 {
    pub order_id: String,
    pub total_amount: Money,
}

/// The shape the domain folds today.
pub type OrderPlaced = OrderPlacedV2;

/// Every shape `OrderPlaced` was ever stored in. Which one comes off the wire is decided by stored data, not
/// by this build — so an older deployment reading a newer envelope during a rolling upgrade is an expected
/// occurrence, and an unknown version comes back as an `Err`, never a panic.
#[derive(Debug, Deserialize)]
#[serde(tag = "schema_version")]
pub enum StoredOrderPlaced {
    #[serde(rename = "1")]
    V1(OrderPlacedV1),
    #[serde(rename = "2")]
    V2(OrderPlacedV2),
}

/// Maps the original flat shape onto the current nested one; only structure changes, never meaning.
impl From<OrderPlacedV1> for OrderPlacedV2 {
    fn from(event: OrderPlacedV1) -> Self {
        OrderPlacedV2 {
            order_id: event.order_id,
            total_amount: Money { minor_units: event.total_minor_units, currency: event.currency },
        }
    }
}

/// Upcasts whichever shape the event was persisted as forward to the current one.
pub fn upcast_order_placed(stored: StoredOrderPlaced) -> OrderPlaced {
    match stored {
        StoredOrderPlaced::V1(v1) => v1.into(),
        StoredOrderPlaced::V2(v2) => v2,
    }
}
```
