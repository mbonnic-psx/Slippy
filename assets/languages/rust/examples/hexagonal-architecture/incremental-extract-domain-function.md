```rust
// deduct_balance.rs — extracted pure function

/// A business outcome, not an error — the pure rule below never panics for an expected rejection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeductRejection {
    NonPositiveAmount,
    CurrencyMismatch,
    InsufficientBalance,
}

/// A pure domain rule: no I/O, no panics for expected outcomes.
pub fn deduct_balance(user: User, amount: Money) -> Result<User, DeductRejection> {
    if amount.minor_units <= 0 {
        return Err(DeductRejection::NonPositiveAmount);
    }
    if amount.currency != user.balance.currency {
        return Err(DeductRejection::CurrencyMismatch);
    }
    if amount.minor_units > user.balance.minor_units {
        return Err(DeductRejection::InsufficientBalance);
    }
    let balance = Money { minor_units: user.balance.minor_units - amount.minor_units, ..user.balance };
    Ok(User { balance, ..user })
}
```
