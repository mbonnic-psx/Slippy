```rust
// materialisation: live — the view is folded per query and nothing is stored. No table, no checkpoint, no
// subscription, no rebuild path, and strongly consistent by construction: it reads the log the command
// just wrote. That is why it is the answer taken by default, and why the slice has to declare the ceiling
// it holds inside. Here the ceiling is asserted rather than assumed.

/// `liveBudget.events` for this view, with the argument for it beside the number: one account stream, which
/// ends at closure, so the ceiling is the busiest account's lifetime and not a guess about traffic. Raise it
/// deliberately, or materialise the view — not because a test went red.
pub const MAX_EVENTS_FOLDED: usize = 40;

#[derive(Debug, thiserror::Error)]
pub enum ReadViewError {
    /// A stream longer than this view may fold on every query.
    #[error(
        "balance view outgrew its live budget: folded {folded} events for {account:?} over a budget of \
         {MAX_EVENTS_FOLDED}, so close the stream at a business boundary or materialise the view"
    )]
    OutgrewItsBudget { folded: usize, account: StreamId },
    #[error(transparent)]
    Corrupt(#[from] CorruptProjection),
    #[error(transparent)]
    Store(#[from] StoreError),
}

pub async fn read_balance_view(store: &impl EventStore, account: &StreamId) -> Result<BalanceView, ReadViewError> {
    let history = store.read_envelopes(account).await?;
    if history.len() > MAX_EVENTS_FOLDED {
        // Failing is the point. A per-query fold does not degrade visibly — it gets slower by a millisecond a
        // week until a handler times out, and by then the fix is a table, a checkpoint, a backfill and every
        // caller. This turns that into one red test on the day the model changed.
        return Err(ReadViewError::OutgrewItsBudget { folded: history.len(), account: account.clone() });
    }
    let mut view = EMPTY_BALANCE_VIEW;
    for committed in &history {
        let envelope = AccountProjectionEnvelope {
            stream_id: committed.stream_id.clone(),
            global_position: committed.global_position,
            data: to_domain_event(committed)?,
        };
        view = apply_to_balance_view(view, &envelope)?;
    }
    Ok(view)
}
```

```rust
// the test that makes the ceiling a fact
#[tokio::test]
async fn a_stream_past_the_budget_fails_rather_than_getting_slower() {
    let store = InMemoryEventStore::default();
    let account = open_account_with_deposits(&store, MAX_EVENTS_FOLDED).await;

    deposit(&store, &account, 1).await;

    let result = read_balance_view(&store, &account).await;
    assert!(matches!(result, Err(ReadViewError::OutgrewItsBudget { .. })), "{result:?}");
}
```
