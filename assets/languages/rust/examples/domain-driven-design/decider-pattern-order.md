```rust
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderItem {
    pub sku: String,
    pub quantity: u32,
}

/// The closed set of intents the Decider accepts.
#[derive(Debug, Clone, PartialEq)]
pub enum OrderCommand {
    Place,
    Ship { tracking_number: String },
}

/// The closed set of facts the Decider produces.
#[derive(Debug, Clone, PartialEq)]
pub enum OrderEvent {
    OrderPlaced { items: Vec<OrderItem>, placed_at: OffsetDateTime },
    OrderShipped { tracking_number: String },
}

/// Each lifecycle phase carries exactly the fields valid in it — illegal states are unrepresentable.
#[derive(Debug, Clone, PartialEq)]
pub enum OrderState {
    Draft { items: Vec<OrderItem> },
    Placed { items: Vec<OrderItem>, placed_at: OffsetDateTime },
    Shipped { items: Vec<OrderItem>, placed_at: OffsetDateTime, tracking_number: String },
}

/// Why `decide` refused — a specific, named reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderRejection {
    OrderNotDraft,
    OrderNotPlaced,
}

/// Step 1: command + current state → an explicit acceptance or rejection.
pub fn decide(command: &OrderCommand, state: &OrderState, now: OffsetDateTime) -> Result<Vec<OrderEvent>, OrderRejection> {
    match (command, state) {
        (OrderCommand::Place, OrderState::Draft { items }) => Ok(vec![OrderEvent::OrderPlaced { items: items.clone(), placed_at: now }]),
        (OrderCommand::Place, _) => Err(OrderRejection::OrderNotDraft),
        (OrderCommand::Ship { tracking_number }, OrderState::Placed { .. }) => {
            Ok(vec![OrderEvent::OrderShipped { tracking_number: tracking_number.clone() }])
        }
        (OrderCommand::Ship { .. }, _) => Err(OrderRejection::OrderNotPlaced),
    }
}

/// Step 2: state + event → new state (a pure transformation). An event that cannot follow the state is
/// corrupt history, and here it panics: the Decider owns its stream, so this is a bug, not an outcome.
pub fn evolve(state: OrderState, event: &OrderEvent) -> OrderState {
    match (state, event) {
        (OrderState::Draft { .. }, OrderEvent::OrderPlaced { items, placed_at }) => {
            OrderState::Placed { items: items.clone(), placed_at: *placed_at }
        }
        (OrderState::Placed { items, placed_at }, OrderEvent::OrderShipped { tracking_number }) => {
            OrderState::Shipped { items, placed_at, tracking_number: tracking_number.clone() }
        }
        (state, event) => panic!("corrupt order history: {event:?} cannot follow {state:?}"),
    }
}

/// Step 3: the starting point for every new order.
pub fn initial_state() -> OrderState {
    OrderState::Draft { items: Vec::new() }
}
```
