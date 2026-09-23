```rust
use thiserror::Error;

/// An ISO 4217 currency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    Gbp,
    Eur,
    Usd,
}

/// An amount in integer minor units (pence, cents) — never a float, so no rounding error can creep into a
/// balance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money {
    pub minor_units: i64,
    pub currency: Currency,
}

impl Money {
    fn is_positive(self) -> bool {
        self.minor_units > 0
    }
}

/// The Decider's state. A balance exists only once the account is open, so the type says so: there is no
/// balance to misread on an account that was never opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountState {
    Unopened,
    Open { balance: Money },
}

pub const INITIAL_STATE: AccountState = AccountState::Unopened;

/// The closed set of commands the account Decider accepts.
#[derive(Debug, Clone, PartialEq)]
pub enum AccountCommand {
    OpenAccount { currency: Currency },
    DepositMoney { amount: Money },
    WithdrawMoney { amount: Money },
}

/// The closed set of facts the account Decider can record.
#[derive(Debug, Clone, PartialEq)]
pub enum AccountEvent {
    AccountOpened { currency: Currency },
    MoneyDeposited { amount: Money },
    MoneyWithdrawn { amount: Money },
}

/// Why `decide` refused a command — a business outcome, returned, never a panic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    AlreadyOpen,
    NotOpen,
    InvalidAmount,
    CurrencyMismatch,
    BalanceOverflow,
    InsufficientFunds,
}

/// What `decide` came to: the events to append, or why not.
pub type Decision = Result<Vec<AccountEvent>, Rejection>;

/// Answers "what should happen?" for a command against the current state. Pure: it returns the events to
/// append or a rejection, and never mutates state or talks to infrastructure. The `match` is exhaustive, so
/// a new command that is not handled here does not compile.
pub fn decide(command: &AccountCommand, state: AccountState) -> Decision {
    use AccountCommand::*;
    match (command, state) {
        (OpenAccount { .. }, AccountState::Open { .. }) => Err(Rejection::AlreadyOpen),
        (OpenAccount { currency }, AccountState::Unopened) => {
            Ok(vec![AccountEvent::AccountOpened { currency: *currency }])
        }
        (DepositMoney { .. } | WithdrawMoney { .. }, AccountState::Unopened) => Err(Rejection::NotOpen),
        (DepositMoney { amount }, AccountState::Open { balance }) => {
            check_amount(*amount, balance)?;
            add_minor_units(balance.minor_units, amount.minor_units).map_err(|_| Rejection::BalanceOverflow)?;
            Ok(vec![AccountEvent::MoneyDeposited { amount: *amount }])
        }
        (WithdrawMoney { amount }, AccountState::Open { balance }) => {
            check_amount(*amount, balance)?;
            if amount.minor_units > balance.minor_units {
                return Err(Rejection::InsufficientFunds);
            }
            Ok(vec![AccountEvent::MoneyWithdrawn { amount: *amount }])
        }
    }
}

fn check_amount(amount: Money, balance: Money) -> Result<(), Rejection> {
    if !amount.is_positive() {
        return Err(Rejection::InvalidAmount);
    }
    if amount.currency != balance.currency {
        return Err(Rejection::CurrencyMismatch);
    }
    Ok(())
}

/// A known event that could not have followed the current state during replay. A corrupted or mis-ordered
/// stream is a real operational occurrence the caller must handle — so it is an `Err`, never a panic.
#[derive(Debug, Error, PartialEq)]
#[error("corrupt account stream: {event:?} cannot follow {state:?}")]
pub struct CorruptHistory {
    pub state: AccountState,
    pub event: AccountEvent,
}

/// Adds a signed delta to a minor-units balance, refusing overflow and a negative result — the accounting
/// invariants a corrupted or buggy delta could otherwise break silently.
fn add_minor_units(balance: i64, delta: i64) -> Result<i64, &'static str> {
    let result = balance.checked_add(delta).ok_or("minor-units overflow")?;
    if result < 0 {
        return Err("negative resulting balance");
    }
    Ok(result)
}

/// Applies a fact to the state. An event that is structurally known but cannot legally follow the current
/// state — a second `AccountOpened`, a withdrawal that overdraws — is corrupt history, reported rather than
/// ignored or panicked on mid-replay.
pub fn evolve(state: AccountState, event: &AccountEvent) -> Result<AccountState, CorruptHistory> {
    let corrupt = || CorruptHistory { state, event: event.clone() };
    match (state, event) {
        (AccountState::Unopened, AccountEvent::AccountOpened { currency }) => {
            Ok(AccountState::Open { balance: Money { minor_units: 0, currency: *currency } })
        }
        (AccountState::Open { balance }, AccountEvent::MoneyDeposited { amount })
            if amount.is_positive() && amount.currency == balance.currency =>
        {
            let minor_units = add_minor_units(balance.minor_units, amount.minor_units).map_err(|_| corrupt())?;
            Ok(AccountState::Open { balance: Money { minor_units, ..balance } })
        }
        (AccountState::Open { balance }, AccountEvent::MoneyWithdrawn { amount })
            if amount.is_positive() && amount.currency == balance.currency =>
        {
            let minor_units = add_minor_units(balance.minor_units, -amount.minor_units).map_err(|_| corrupt())?;
            Ok(AccountState::Open { balance: Money { minor_units, ..balance } })
        }
        _ => Err(corrupt()),
    }
}
```
