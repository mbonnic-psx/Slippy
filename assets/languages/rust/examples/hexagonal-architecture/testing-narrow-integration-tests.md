```rust
// Real database — a fresh one per test, no shared state
#[tokio::test]
async fn sql_order_repository_round_trips_an_order_through_persistence() {
    let orders = SqlOrderRepository::new(test_db().await);
    let order = test_order();

    orders.save(&order).await.expect("save");
    let found = orders.find_by_id(&order.id).await.expect("find");

    assert_eq!(found, Some(order));
}

// Real HTTP against a local server
#[tokio::test]
async fn stripe_payment_gateway_succeeds_on_a_valid_charge() {
    let app = Router::new().route("/v1/charges", post(|| async { Json(json!({ "id": "ch_123", "status": "succeeded" })) }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(axum::serve(listener, app).into_future());

    let gateway = StripePaymentGateway::new(&base_url, "sk_test");
    let result = gateway.charge(test_amount(), &test_payment_info()).await.expect("charge");

    assert!(result.success);
}
```
