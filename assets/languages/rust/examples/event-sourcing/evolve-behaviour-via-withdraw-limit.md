```rust
// Proves the fold indirectly, by which withdrawals `decide` accepts against the resulting state: the running
// balance is never asserted directly, only the behaviour it produces at its boundary.
#[test]
fn the_fold_sets_the_withdrawal_limit() {
    let state = [opened(), deposited(10_000), withdrawn(3_000)]
        .iter()
        .try_fold(INITIAL_STATE, evolve)
        .expect("a valid history");

    for (name, withdrawal, accepted) in [
        ("withdrawing exactly the available balance is accepted", 7_000, true),
        ("withdrawing one minor unit over the available balance is rejected", 7_001, false),
    ] {
        let decision = decide(&AccountCommand::WithdrawMoney { amount: money(withdrawal) }, state);

        assert_eq!(decision.is_ok(), accepted, "{name}");
    }
}
```
