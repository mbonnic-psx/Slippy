```rust
/// The driving port — exposed by the application, called by driving adapters — for tax on an amount.
pub trait ForCalculatingTaxes {
    fn tax_on(&self, amount: Money) -> Money;
}

// The TDD "fake it" step: `TaxCalculation::new()` (defined elsewhere) returns a trivial implementation that
// answers with a hard-coded placeholder.
#[test]
fn returns_the_flat_placeholder_tax() {
    let calculator = TaxCalculation::new();

    assert_eq!(calculator.tax_on(Money::new(100, Currency::Gbp)), Money::new(0, Currency::Gbp));
}
```
