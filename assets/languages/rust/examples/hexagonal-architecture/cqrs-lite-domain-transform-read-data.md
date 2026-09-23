```rust
/// The business rule for dashboard urgency: fewer days remaining than this counts as urgent.
pub const URGENT_WITHIN_DAYS: i64 = 30;

/// The display-ready DTO for the dashboard view.
#[derive(Debug, PartialEq)]
pub struct DashboardCard {
    pub title: String,
    pub emoji: String,
    pub days_away: i64,
    pub savings: SavingsDisplay,
    pub is_urgent: bool,
}

/// A pure, provider-free transform: no ports, no I/O. `DashboardRow` is the raw joined row the driven query
/// adapter produced.
///
/// `is_urgent` is a genuine business rule — a threshold on days until the event — so it belongs in the
/// hexagon. Cosmetic formatting is out of scope; `build_savings_display` is a separate concern at the edge.
pub fn to_dashboard_card(row: DashboardRow, now: time::OffsetDateTime) -> DashboardCard {
    let days_away = (row.event_date - now).whole_days();
    DashboardCard {
        savings: build_savings_display(row.saved_amount, row.target_amount, now),
        title: row.event_title,
        emoji: row.occasion_emoji,
        days_away,
        is_urgent: days_away < URGENT_WITHIN_DAYS,
    }
}
```
