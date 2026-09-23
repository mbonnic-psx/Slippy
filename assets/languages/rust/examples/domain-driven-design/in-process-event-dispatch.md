```rust
// --- minimal supporting domain types (the Decider itself lives elsewhere in the skill) ---

#[derive(Debug, Clone, PartialEq)]
pub enum OrderState {
    Draft { items: Vec<OrderItem> },
    Placed { items: Vec<OrderItem> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderEvent {
    OrderPlaced { items: Vec<OrderItem>, placed_at: OffsetDateTime },
}

pub fn decide(state: &OrderState, now: OffsetDateTime) -> Result<Vec<OrderEvent>, &'static str> {
    match state {
        OrderState::Draft { items } => Ok(vec![OrderEvent::OrderPlaced { items: items.clone(), placed_at: now }]),
        _ => Err("order-not-draft"),
    }
}

pub fn evolve(_state: OrderState, event: &OrderEvent) -> OrderState {
    match event {
        OrderEvent::OrderPlaced { items, .. } => OrderState::Placed { items: items.clone() },
    }
}

// --- the pattern this file actually illustrates ---

#[derive(Debug, Clone, PartialEq)]
pub struct StoredOrder {
    pub state: OrderState,
    pub version: i64,
}

#[derive(Debug, PartialEq)]
pub enum PlaceOrderResult {
    Placed(OrderState),
    Refused(&'static str),
}

/// The application-owned persistence port.
pub trait OrderRepository {
    fn find_by_id(&self, id: &OrderId) -> Result<Option<StoredOrder>, RepositoryError>;
    fn save(&self, state: &OrderState, expected_version: i64) -> Result<SaveOutcome, RepositoryError>;
}

/// The application-owned port for dispatching domain events.
pub trait OrderNotifier {
    fn notify(&self, event: &OrderEvent) -> Result<(), RepositoryError>;
}

pub fn handle_place_order(
    orders: &impl OrderRepository,
    notifier: &impl OrderNotifier,
    order_id: &OrderId,
    now: OffsetDateTime,
) -> Result<PlaceOrderResult, RepositoryError> {
    let Some(stored) = orders.find_by_id(order_id)? else {
        return Ok(PlaceOrderResult::Refused("not-found"));
    };
    let events = match decide(&stored.state, now) {
        Ok(events) => events,
        Err(reason) => return Ok(PlaceOrderResult::Refused(reason)),
    };
    let new_state = events.iter().fold(stored.state, evolve);

    if orders.save(&new_state, stored.version)? == SaveOutcome::Conflict {
        return Ok(PlaceOrderResult::Refused("concurrent-change"));
    }

    // Dispatch in-process — simple, but not durable.
    for event in &events {
        notifier.notify(event)?;
    }
    Ok(PlaceOrderResult::Placed(new_state))
}
```
