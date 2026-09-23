```rust
/// A read-optimised row: an account's events projected into the shape a balance enquiry wants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BalanceView {
    Unopened,
    Open { account_id: StreamId, balance_minor_units: i64, currency: Currency },
}

/// The seed the fold starts from.
pub const EMPTY_BALANCE_VIEW: BalanceView = BalanceView::Unopened;

/// An account event with the metadata a projection needs and the domain event does not carry: which stream
/// it belongs to, and where it sits in the store-wide order.
#[derive(Debug, Clone, PartialEq)]
pub struct AccountProjectionEnvelope {
    pub stream_id: StreamId,
    pub global_position: u64,
    pub data: AccountEvent,
}

/// An event that could not have followed the projection's current state — a corrupted or mis-ordered stream,
/// which the caller handles rather than the fold panicking mid-way.
#[derive(Debug, thiserror::Error, PartialEq)]
#[error("corrupt balance projection: {event:?} cannot follow {view:?}")]
pub struct CorruptProjection {
    pub view: BalanceView,
    pub event: AccountEvent,
}

/// Folds one envelope into the running view.
pub fn apply(view: BalanceView, envelope: &AccountProjectionEnvelope) -> Result<BalanceView, CorruptProjection> {
    let corrupt = || CorruptProjection { view: view.clone(), event: envelope.data.clone() };
    match (&view, &envelope.data) {
        (BalanceView::Unopened, AccountEvent::AccountOpened { currency }) => Ok(BalanceView::Open {
            account_id: envelope.stream_id.clone(),
            balance_minor_units: 0,
            currency: *currency,
        }),
        (BalanceView::Open { account_id, balance_minor_units, currency }, event) if *account_id == envelope.stream_id => {
            let delta = match event {
                AccountEvent::MoneyDeposited { amount } if amount.minor_units > 0 && amount.currency == *currency => {
                    amount.minor_units
                }
                AccountEvent::MoneyWithdrawn { amount }
                    if amount.minor_units > 0
                        && amount.currency == *currency
                        && amount.minor_units <= *balance_minor_units =>
                {
                    -amount.minor_units
                }
                _ => return Err(corrupt()),
            };
            let balance = balance_minor_units.checked_add(delta).filter(|b| *b >= 0).ok_or_else(corrupt)?;
            Ok(BalanceView::Open { account_id: account_id.clone(), balance_minor_units: balance, currency: *currency })
        }
        _ => Err(corrupt()),
    }
}
```
