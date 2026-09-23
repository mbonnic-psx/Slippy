```rust
/// The flat, read-optimised shape for the dashboard view.
#[derive(Debug, sqlx::FromRow)]
pub struct DashboardRow {
    pub event_title: String,
    pub occasion_emoji: String,
    pub event_date: time::OffsetDateTime,
    pub saved_amount: Option<i64>,
    pub target_amount: i64,
    pub recipient_name: String,
}

/// A driven adapter query — not a repository. It joins across several aggregates for a read-only dashboard,
/// deliberately bypassing the aggregate-per-repository pattern because this is a display read, not a write.
pub async fn dashboard_cards(pool: &sqlx::PgPool, user_id: &str) -> Result<Vec<DashboardRow>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            events.title                 AS event_title,
            occasions.emoji              AS occasion_emoji,
            events.event_date            AS event_date,
            savings_goals.saved_amount   AS saved_amount,
            savings_goals.target_amount  AS target_amount,
            recipients.name              AS recipient_name
        FROM events
        INNER JOIN occasions    ON occasions.id = events.occasion_id
        LEFT JOIN savings_goals ON savings_goals.event_id = events.id
        INNER JOIN recipients   ON recipients.id = events.recipient_id
        WHERE events.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}
```
