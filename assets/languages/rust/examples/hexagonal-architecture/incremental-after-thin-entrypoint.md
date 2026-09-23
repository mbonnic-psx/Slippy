```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct DeductBody {
    amount_minor_units: i64,
    currency: Currency,
}

/// A serverless-style executable entry point: inline composition plus the driving adapter. Valid only while
/// this object graph stays trivial and unshared.
pub async fn deduct(
    State(env): State<Env>,
    principal: AuthenticatedPrincipal, // authentication owns the principal; the body cannot select a user
    body: Result<Json<DeductBody>, JsonRejection>,
) -> Response {
    let pool = open_pool(&env.database_url).await;

    // Wire adapters.
    let users = PostgresUserRepository::new(pool);
    let balance_deduction = UserBalanceDeduction { users };

    // Translate transport syntax separately from request-schema validation.
    let Ok(Json(body)) = body else {
        return json_error(StatusCode::BAD_REQUEST, "malformed-json");
    };

    // Call the use case.
    let command = DeductUserBalanceCommand {
        principal,
        amount: Money { minor_units: body.amount_minor_units, currency: body.currency },
    };
    let outcome = match balance_deduction.deduct_user_balance(command).await {
        Ok(outcome) => outcome,
        Err(_) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal-error"),
    };
    match outcome {
        DeductUserBalanceOutcome::Succeeded(user) => Json(json!({ "balance": user.balance })).into_response(),
        failure => {
            let (status, reason) = deduct_failure(failure).expect("every other outcome is a failure");
            json_error(status, reason)
        }
    }
}

fn deduct_failure(outcome: DeductUserBalanceOutcome) -> Option<(StatusCode, &'static str)> {
    use DeductUserBalanceOutcome::*;
    match outcome {
        Succeeded(_) => None,
        NotFound => Some((StatusCode::NOT_FOUND, "not-found")),
        ConcurrentChange => Some((StatusCode::CONFLICT, "concurrent-change")),
        NonPositiveAmount => Some((StatusCode::UNPROCESSABLE_ENTITY, "non-positive-amount")),
        CurrencyMismatch => Some((StatusCode::UNPROCESSABLE_ENTITY, "currency-mismatch")),
        InsufficientBalance => Some((StatusCode::UNPROCESSABLE_ENTITY, "insufficient-balance")),
    }
}

fn json_error(status: StatusCode, reason: &str) -> Response {
    (status, Json(json!({ "error": reason }))).into_response()
}
```
