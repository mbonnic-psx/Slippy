```rust
// Repository against a real database — a fresh schema per test.
#[tokio::test]
async fn sql_order_repository_persists_and_retrieves_an_order() {
    let pool = test_db().await; // a fresh schema, dropped with the pool
    let orders = SqlOrderRepository::new(pool);
    let order = test_order();

    orders.save(&order).await.expect("save");
    let found = orders.find_by_id(&order.id).await.expect("find");

    assert_eq!(found, Some(order));
}
```
