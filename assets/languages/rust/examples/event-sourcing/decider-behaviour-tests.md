```rust
// Event and money factories: complete, valid data, per the testing skill's factory pattern.
fn money(minor_units: i64) -> Money {
    Money { minor_units, currency: Currency::Gbp }
}

fn opened() -> AccountEvent {
    AccountEvent::AccountOpened { currency: Currency::Gbp }
}

fn deposited(minor_units: i64) -> AccountEvent {
    AccountEvent::MoneyDeposited { amount: money(minor_units) }
}

fn withdrawn(minor_units: i64) -> AccountEvent {
    AccountEvent::MoneyWithdrawn { amount: money(minor_units) }
}

fn given(history: &[AccountEvent]) -> AccountState {
    history.iter().try_fold(INITIAL_STATE, evolve).expect("a valid history")
}

#[test]
fn records_a_deposit_as_money_deposited_on_an_open_account() {
    let decision = decide(&AccountCommand::DepositMoney { amount: money(5_000) }, given(&[opened()]));

    assert_eq!(decision, Ok(vec![deposited(5_000)]));
}

#[test]
fn rejects_a_withdrawal_that_exceeds_the_balance() {
    let state = given(&[opened(), deposited(5_000)]);

    let decision = decide(&AccountCommand::WithdrawMoney { amount: money(10_000) }, state);

    assert_eq!(decision, Err(Rejection::InsufficientFunds));
}

#[test]
fn rejects_any_operation_on_an_account_that_was_never_opened() {
    let decision = decide(&AccountCommand::DepositMoney { amount: money(5_000) }, given(&[]));

    assert_eq!(decision, Err(Rejection::NotOpen));
}
```
