```rust
/// A serverless-style executable entry point: inline composition plus the driving adapter. Valid only while
/// this object graph stays trivial and unshared.
pub async fn create_order(State(env): State<Env>, body: Result<Json<Value>, JsonRejection>) -> Response {
    let pool = open_pool(&env.database_url).await;

    // Wire adapters.
    let orders = PostgresOrderRepository::new(pool);
    let gateway = StripeGateway::new(&env.stripe_key);
    let order_placement = OrderPlacement::new(orders, gateway);

    // Translate transport syntax separately from request-schema validation.
    let Ok(Json(raw)) = body else {
        return json_error(StatusCode::BAD_REQUEST, "malformed-json");
    };
    let Ok(command) = serde_json::from_value::<CreateOrderCommand>(raw) else {
        return json_error(StatusCode::UNPROCESSABLE_ENTITY, "invalid-body");
    };

    // Call the use case.
    match order_placement.place_order(command.into()).await {
        Ok(result) => Json(result).into_response(),
        Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal-error"),
    }
}

fn json_error(status: StatusCode, reason: &str) -> Response {
    (status, Json(json!({ "error": reason }))).into_response()
}
```
