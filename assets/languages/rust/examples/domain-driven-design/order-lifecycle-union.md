```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderItem {
    pub sku: String,
    pub quantity: u32,
}

/// The lifecycle as an enum: a draft has no placement time, a placed order no tracking number. A field valid
/// only in a later state cannot exist in an earlier one, so there is no zero value to misread and no
/// constructor discipline to remember.
#[derive(Debug, Clone, PartialEq)]
pub enum Order {
    Draft {
        items: Vec<OrderItem>,
    },
    Placed {
        items: Vec<OrderItem>,
        placed_at: OffsetDateTime,
    },
    Shipped {
        items: Vec<OrderItem>,
        placed_at: OffsetDateTime,
        shipped_at: OffsetDateTime,
        tracking_number: String,
    },
}
```
