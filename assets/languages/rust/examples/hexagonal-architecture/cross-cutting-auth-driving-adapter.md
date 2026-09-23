```rust
/// A driving adapter: extract and validate authentication, then delegate to the use case.
pub async fn pledge_to_occasion(
    State(app): State<App>,
    principal: AuthenticatedPledger, // an extractor: the sole production constructor, or a 401
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    let Ok(Json(raw)) = body else {
        return json_error(StatusCode::BAD_REQUEST, "malformed-json");
    };
    let Ok(body) = serde_json::from_value::<PledgeBody>(raw) else {
        return json_error(StatusCode::UNPROCESSABLE_ENTITY, "invalid-body");
    };

    // The strict body has no actor or tenant fields; the principal owns attribution.
    match app.pledging.pledge_to_occasion(body.into_command(principal)).await {
        Ok(result) => write_result(result),
        Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal-error"),
    }
}
```
