```rust
fn envelope(data: AccountEvent, global_position: u64) -> AccountProjectionEnvelope {
    AccountProjectionEnvelope { stream_id: StreamId("acc-1".into()), global_position, data }
}

fn fold(envelopes: &[AccountProjectionEnvelope]) -> Result<BalanceView, CorruptProjection> {
    envelopes.iter().try_fold(EMPTY_BALANCE_VIEW, apply)
}

#[test]
fn reflects_the_net_balance_of_a_sequence_of_account_events() {
    let view = fold(&[envelope(opened(), 1), envelope(deposited(10_000), 2), envelope(withdrawn(3_000), 3)]);

    assert_eq!(
        view,
        Ok(BalanceView::Open {
            account_id: StreamId("acc-1".into()),
            currency: Currency::Gbp,
            balance_minor_units: 7_000,
        })
    );
}

#[test]
fn a_duplicate_account_opening_is_corrupt_projection_history() {
    let reopened = AccountEvent::AccountOpened { currency: Currency::Eur };

    let view = fold(&[envelope(opened(), 1), envelope(reopened, 2)]);

    assert!(matches!(view, Err(CorruptProjection { .. })), "{view:?}");
}
```
