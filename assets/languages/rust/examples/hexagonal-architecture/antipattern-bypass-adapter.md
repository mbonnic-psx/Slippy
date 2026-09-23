```rust
// ❌ Route handler hits the database directly
async fn list_active_users(State(app): State<App>) -> Result<Json<Vec<UserRow>>, StatusCode> {
    let rows = sqlx::query_as::<_, UserRow>("SELECT id, email FROM users WHERE active = true")
        .fetch_all(&app.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows))
}

// ✅ Route handler calls a use case, which goes through a port
async fn list_active_users(State(app): State<App>) -> Result<Json<Vec<User>>, StatusCode> {
    let users = app.get_active_users.execute().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(users))
}
```
