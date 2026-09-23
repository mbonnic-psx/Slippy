```rust
// load_participant_view.rs
pub async fn load_participant_view(
    db: &impl Items,
    event_id: EventId,
    user_id: UserId,
) -> Result<ParticipantView, LoadError> {
    let items = db.items_for(event_id, user_id).await?;

    let mut view = ParticipantView::default();
    for item in items {
        if item.is_claimed && item.is_claimed_by_current_user {
            view.your_claims.push(item);
        } else if !item.is_claimed_by_current_user {
            view.available.push(item);
        }
    }
    Ok(view)
}

// The behavioural test for load_participant_view covers the filtering:
#[tokio::test]
async fn load_participant_view_splits_claimed_and_available() {
    let result = load_participant_view(&db, event_id, user_id).await.unwrap();

    assert_eq!(result.your_claims.len(), 1);
    assert_eq!(result.available.len(), 2);
}
```
