```rust
// ❌ Business rule in the route handler
async fn approve_order(State(app): State<App>, Path(order_id): Path<String>) -> impl IntoResponse {
    let order = app.orders.find_by_id(&order_id).await.unwrap();
    if order.item_count > 100 {
        require_manager_approval(&order); // business rule!
    }
    // ...
}

// ✅ Business rule in the domain
pub enum PlaceOrderResult {
    Placed(Order),
    RequiresApproval,
}

pub fn place_order(order: Order) -> PlaceOrderResult {
    if order.item_count > 100 {
        return PlaceOrderResult::RequiresApproval;
    }
    PlaceOrderResult::Placed(order)
}
```
