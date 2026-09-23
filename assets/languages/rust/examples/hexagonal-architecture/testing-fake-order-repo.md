```rust
use std::cell::RefCell;
use std::collections::HashMap;

/// Returned by any `OrderRepository` — fake or real — when no order matches the id.
#[derive(Debug, thiserror::Error, PartialEq)]
#[error("order not found")]
pub struct OrderNotFound;

/// An in-memory `OrderRepository` for use-case tests. It implements the same trait a real adapter does, and
/// exposes `saved` so tests can assert on what was actually saved.
#[derive(Default)]
pub struct FakeOrderRepository {
    orders: RefCell<HashMap<OrderId, Order>>,
    pub saved: RefCell<Vec<Order>>,
}

impl OrderRepository for FakeOrderRepository {
    fn find_by_id(&self, id: &OrderId) -> Result<Order, OrderNotFound> {
        self.orders.borrow().get(id).cloned().ok_or(OrderNotFound)
    }

    fn save(&self, order: &Order) -> Result<(), OrderNotFound> {
        self.orders.borrow_mut().insert(order.id.clone(), order.clone());
        self.saved.borrow_mut().push(order.clone());
        Ok(())
    }
}
```
