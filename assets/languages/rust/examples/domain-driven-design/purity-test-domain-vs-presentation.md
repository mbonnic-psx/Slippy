```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GiftItemStatus {
    Idea,
    Selected,
    Purchased,
}

/// A minimal stand-in for the sibling type `calculate_committed_total` reads.
#[derive(Debug, Clone, PartialEq)]
pub struct GiftItem {
    pub status: GiftItemStatus,
    pub price_pence: i64,
}

/// Pure but NOT domain — it formats for human display. It belongs in presentation code; a hexagonal app may
/// place it at the driving edge.
pub fn format_event_date(event_date: Option<Date>) -> String {
    let format = time::macros::format_description!("[month repr:long] [day padding:none], [year]");
    event_date.map(|date| date.format(&format).expect("a static format")).unwrap_or_default()
}

/// Pure AND domain — a business rule that affects behaviour. The application supplies `now` as data; domain
/// code does not read a clock.
pub fn is_past_event(event_date: Option<OffsetDateTime>, now: OffsetDateTime) -> bool {
    event_date.is_some_and(|date| date < now)
}

/// Pure AND domain — a business calculation. It belongs in domain policy for budgets.
pub fn calculate_committed_total(items: &[GiftItem]) -> i64 {
    items.iter().filter(|item| item.status != GiftItemStatus::Idea).map(|item| item.price_pence).sum()
}
```
