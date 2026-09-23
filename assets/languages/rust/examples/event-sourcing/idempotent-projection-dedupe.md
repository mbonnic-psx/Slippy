```rust
/// A per-account (single-stream) read-model row; a version is unique within it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BalanceRow {
    pub balance_minor_units: i64,
    pub applied_through: u64,
}

/// Applies a deposit idempotently: an event at or below the version already applied is ignored, so a
/// redelivered event cannot double-count.
pub fn project_row(row: BalanceRow, amount: Money, version: u64) -> Result<BalanceRow, Overflow> {
    if version <= row.applied_through {
        return Ok(row);
    }
    let balance_minor_units = row.balance_minor_units.checked_add(amount.minor_units).ok_or(Overflow)?;
    Ok(BalanceRow { balance_minor_units, applied_through: version })
}

#[test]
fn a_redelivered_event_is_ignored() {
    let amount = money(10_000);

    let once = project_row(BalanceRow::default(), amount, 1).unwrap();
    let twice = project_row(once, amount, 1).unwrap(); // the same event, redelivered at the same version

    assert_eq!(once.balance_minor_units, 10_000);
    assert_eq!(twice.balance_minor_units, 10_000, "not double-counted");
}
```
