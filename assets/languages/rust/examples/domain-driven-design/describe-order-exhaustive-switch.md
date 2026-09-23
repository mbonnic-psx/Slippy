```rust
/// The lifecycle as an enum: each variant carries only the fields valid in it.
#[derive(Debug, Clone, PartialEq)]
pub enum Order {
    Draft { items: Vec<OrderItem> },
    Placed { items: Vec<OrderItem>, placed_at: OffsetDateTime },
    Shipped { items: Vec<OrderItem>, placed_at: OffsetDateTime, shipped_at: OffsetDateTime, tracking_number: String },
}

/// Exhaustively handles every lifecycle variant. There is no default arm: a variant added to `Order` without
/// updating this function does not compile.
pub fn describe_order(order: &Order) -> String {
    match order {
        Order::Draft { items } => format!("Draft with {} items", items.len()),
        Order::Placed { placed_at, .. } => format!("Placed at {placed_at}"),
        Order::Shipped { tracking_number, .. } => format!("Shipped: {tracking_number}"),
    }
}
```
