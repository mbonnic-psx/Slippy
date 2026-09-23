```rust
// prepare_participant_data.rs (new module, one caller)
pub fn prepare_participant_data(items: Vec<Item>) -> ParticipantView {
    let mut view = ParticipantView::default();
    for item in items {
        if item.is_claimed && item.is_claimed_by_current_user {
            view.your_claims.push(item);
        } else if !item.is_claimed_by_current_user {
            view.available.push(item);
        }
    }
    view
}

// ...and its own #[cfg(test)] module, testing the helper directly
#[test]
fn prepare_participant_data_filters_claims() {
    // ...
}
```
