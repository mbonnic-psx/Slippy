```rust
// WRONG — builds its own dependencies inline (untestable, tightly coupled)
async fn create_order_wrong(order: NewOrder) -> Result<(), PlaceError> {
    let orders = PostgresOrderRepository::new(must_open_pool().await); // hard-coded
    let gateway = StripeGateway::new(&std::env::var("STRIPE_KEY")?); // hard-coded
    orders.find_or_create_pending(&order).await?;
    Ok(())
}

// RIGHT — dependencies arrive as constructor parameters

#[derive(Debug, PartialEq)]
pub enum Recorded {
    Recorded(Order),
    Conflict,
}

pub trait OrderRepository {
    async fn find_by_id(&self, order_id: &OrderId) -> Result<Option<Order>, RepositoryError>;
    async fn find_or_create_pending(&self, order: &NewOrder) -> Result<Order, RepositoryError>;
    async fn record_payment(&self, order_id: &OrderId, payment_id: &PaymentId, expected_version: i64)
        -> Result<Recorded, RepositoryError>;
    async fn record_charge(&self, order_id: &OrderId, charge_id: &ChargeId, expected_version: i64)
        -> Result<Recorded, RepositoryError>;
}

/// The business outcome of placing an order. Never used for infrastructure failure, which is the `Err` of the
/// `Result` it travels in.
#[derive(Debug, PartialEq)]
pub enum OrderResult {
    Placed(Order),
    Refused(String),
}

/// The driving port `OrderPlacement` implements.
pub trait ForPlacingOrders {
    async fn place_order(&self, order: NewOrder) -> Result<OrderResult, PlaceError>;
}

/// The use case. Its dependencies arrive as constructor parameters — never built inside a method.
pub struct OrderPlacement<R, G> {
    orders: R,
    gateway: G,
}

impl<R: OrderRepository, G: PaymentGateway> OrderPlacement<R, G> {
    pub fn new(orders: R, gateway: G) -> Self {
        Self { orders, gateway }
    }
}

impl<R: OrderRepository, G: PaymentGateway> ForPlacingOrders for OrderPlacement<R, G> {
    async fn place_order(&self, order: NewOrder) -> Result<OrderResult, PlaceError> {
        let concurrent = || Ok(OrderResult::Refused("concurrent-change".into()));
        let mut payable = self.orders.find_or_create_pending(&order).await?;
        if payable.status == OrderStatus::Paid {
            return Ok(OrderResult::Placed(payable));
        }

        if payable.payment_id.is_none() {
            let payment_id = match self.gateway.prepare_payment(payable.total, &payable.id, &payable.id).await? {
                PreparePaymentResult::Prepared { payment_id } => payment_id,
                PreparePaymentResult::Failed { reason } => return Ok(OrderResult::Refused(reason.to_string())),
            };
            payable = match self.orders.record_payment(&payable.id, &payment_id, payable.version).await? {
                Recorded::Recorded(order) => order,
                Recorded::Conflict => match self.orders.find_by_id(&payable.id).await? {
                    Some(current) if current.status == OrderStatus::Paid => return Ok(OrderResult::Placed(current)),
                    Some(current) if current.status == OrderStatus::Pending && current.payment_id.is_some() => current,
                    _ => return concurrent(),
                },
            };
        }

        let Some(payment_id) = payable.payment_id.clone() else {
            return concurrent();
        };
        let charge_id = match self.gateway.complete_payment(&payment_id, &payable.payment).await? {
            PaymentOutcome::Paid { charge_id } => charge_id,
            PaymentOutcome::Pending => return Ok(OrderResult::Refused("payment-pending".into())),
            PaymentOutcome::Declined { reason } => return Ok(OrderResult::Refused(reason.to_string())),
        };

        match self.orders.record_charge(&payable.id, &charge_id, payable.version).await? {
            Recorded::Recorded(order) => Ok(OrderResult::Placed(order)),
            Recorded::Conflict => match self.orders.find_by_id(&payable.id).await? {
                Some(current) if current.status == OrderStatus::Paid && current.charge_id.as_ref() == Some(&charge_id) => {
                    Ok(OrderResult::Placed(current))
                }
                _ => concurrent(),
            },
        }
    }
}
```
