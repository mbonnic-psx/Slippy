```rust
/// A driving adapter: it logs the request and response cycle.
pub async fn pledge_to_occasion(State(app): State<App>, body: Result<Json<Value>, JsonRejection>) -> Response {
    let Ok(Json(raw)) = body else {
        return json_error(StatusCode::BAD_REQUEST, "malformed-json");
    };
    let Ok(body) = serde_json::from_value::<PledgeBody>(raw) else {
        return json_error(StatusCode::UNPROCESSABLE_ENTITY, "invalid-body");
    };
    let occasion_id = body.occasion_id.clone();

    match app.pledging.pledge_to_occasion(body.into_command()).await {
        Ok(result) => {
            if let Err(reason) = &result {
                tracing::warn!(?reason, %occasion_id, "pledge rejected");
            }
            write_result(result)
        }
        Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal-error"),
    }
}

/// A driven adapter: it logs its infrastructure interactions.
pub struct OccasionRepository {
    pool: sqlx::PgPool,
}

impl OccasionRepository {
    pub async fn save(&self, occasion: &Occasion) -> Result<(), sqlx::Error> {
        sqlx::query(INSERT_OCCASION).bind(&occasion.id).bind(&occasion.name).execute(&self.pool).await?;
        tracing::debug!(id = %occasion.id, "occasion saved");
        Ok(())
    }
}
```
