```rust
use time::macros::datetime;

#[test]
fn is_past_event() {
    let now = datetime!(2026-03-20 12:00 UTC);
    for (name, event_date, past) in [
        ("a date before now is past", datetime!(2026-03-19 12:00 UTC), true),
        ("a date after now is not past", datetime!(2026-03-21 12:00 UTC), false),
    ] {
        assert_eq!(is_past_event(Some(event_date), now), past, "{name}");
    }
}

#[test]
fn committed_total_includes_only_items_that_are_not_ideas() {
    let items = [test_item(GiftItemStatus::Selected, 5_000), test_item(GiftItemStatus::Idea, 3_000)];

    assert_eq!(calculate_committed_total(&items), 5_000);
}

/// A local test-data helper for the sibling `GiftItem` type.
fn test_item(status: GiftItemStatus, price_pence: i64) -> GiftItem {
    GiftItem { status, price_pence }
}
```
