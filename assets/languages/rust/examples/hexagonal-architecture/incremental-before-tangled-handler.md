```rust
/// The BEFORE state — everything crammed into the route handler.
pub async fn deduct(
    State(app): State<App>,
    principal: AuthenticatedPrincipal,
    body: Result<Json<DeductBody>, JsonRejection>,
) -> Response {
    let Ok(Json(body)) = body else {
        return json_error(StatusCode::BAD_REQUEST, "malformed-json");
    };

    let row = sqlx::query_as::<_, (i64, String)>("SELECT balance_minor_units, currency FROM users WHERE id = $1")
        .bind(&principal.user_id)
        .fetch_optional(&app.pool)
        .await;
    let (balance_minor_units, currency) = match row {
        Ok(Some(row)) => row,
        Ok(None) => return json_error(StatusCode::NOT_FOUND, "not found"),
        Err(_) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal-error"),
    };

    if body.amount_minor_units <= 0 {
        return json_error(StatusCode::UNPROCESSABLE_ENTITY, "invalid amount");
    }
    if body.currency != currency {
        return json_error(StatusCode::UNPROCESSABLE_ENTITY, "currency mismatch");
    }
    if balance_minor_units < body.amount_minor_units {
        return json_error(StatusCode::UNPROCESSABLE_ENTITY, "insufficient");
    }

    let new_balance = balance_minor_units - body.amount_minor_units;
    let updated = sqlx::query("UPDATE users SET balance_minor_units = $1 WHERE id = $2")
        .bind(new_balance)
        .bind(&principal.user_id)
        .execute(&app.pool)
        .await;
    if updated.is_err() {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal-error");
    }

    Json(json!({ "balanceMinorUnits": new_balance, "currency": currency })).into_response()
}

fn json_error(status: StatusCode, reason: &str) -> Response {
    (status, Json(json!({ "error": reason }))).into_response()
}
```
