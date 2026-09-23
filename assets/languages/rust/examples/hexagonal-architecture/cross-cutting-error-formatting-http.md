```rust
/// Maps a rejection to an HTTP status through one exhaustive `match`, not ad hoc if/else — a new reason that
/// is not mapped here does not compile.
fn status_for(reason: PledgeRejection) -> StatusCode {
    match reason {
        PledgeRejection::NotFound => StatusCode::NOT_FOUND,
        PledgeRejection::ConcurrentChange => StatusCode::CONFLICT,
        PledgeRejection::NonPositiveAmount
        | PledgeRejection::CurrencyMismatch
        | PledgeRejection::ExceedsBudget
        | PledgeRejection::FundingClosed => StatusCode::UNPROCESSABLE_ENTITY,
    }
}

/// The driving adapter: it translates a domain or application result into an HTTP response, never the other
/// way round.
pub fn write_result(result: PledgeResult) -> Response {
    match result {
        Ok(occasion) => Json(json!({ "pledged": occasion.total_pledged })).into_response(),
        Err(reason) => (status_for(reason), Json(json!({ "error": reason }))).into_response(),
    }
}
```
