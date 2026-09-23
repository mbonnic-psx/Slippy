```rust
/// Why a withdrawal can be refused — a closed set, not a free string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WithdrawRejection {
    NotOpen,
    InsufficientFunds,
    InvalidAmount,
    CurrencyMismatch,
}

/// The decide slice for `WithdrawMoney`: the account is open, the amount is positive and in the account's
/// own currency, and the balance covers it.
pub fn decide_withdraw(amount: Money, state: AccountState) -> Result<Vec<AccountEvent>, WithdrawRejection> {
    let AccountState::Open { balance } = state else {
        return Err(WithdrawRejection::NotOpen);
    };
    if amount.minor_units <= 0 {
        return Err(WithdrawRejection::InvalidAmount);
    }
    if amount.currency != balance.currency {
        return Err(WithdrawRejection::CurrencyMismatch);
    }
    if amount.minor_units > balance.minor_units {
        return Err(WithdrawRejection::InsufficientFunds);
    }
    Ok(vec![AccountEvent::MoneyWithdrawn { amount }])
}
```
