```rust
/// A read-optimised DTO, not a domain aggregate.
#[derive(Debug, sqlx::FromRow)]
pub struct ParticipantEventView {
    pub event_id: String,
    pub event_name: String,
    pub occasion_name: String,
    pub claimed_by: Option<String>,
}

/// A query function that joins across aggregates for a read-only display view. It lives in
/// `reporting/adapters/driven/postgres/queries/`, outside the repository pattern, and returns a DTO rather
/// than a domain aggregate.
pub async fn participant_event_view(
    pool: &sqlx::PgPool,
    event_id: &str,
) -> Result<Option<ParticipantEventView>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT e.id AS event_id, e.name AS event_name, o.name AS occasion_name, g.claimed_by
        FROM events e
        INNER JOIN occasions o ON o.id = e.occasion_id
        LEFT JOIN gift_claims g ON g.event_id = e.id
        WHERE e.id = $1
        "#,
    )
    .bind(event_id)
    .fetch_optional(pool)
    .await
}
```
