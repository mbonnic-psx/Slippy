```rust
/// A deposit carrying its own timestamp, so `decide` stays a pure function of its arguments — it never
/// reads the clock itself.
#[derive(Debug, Clone, PartialEq)]
pub struct DepositMoneyAt {
    pub amount: Money,
    pub at: time::OffsetDateTime,
}
```
