```rust
/// A driven port — application-owned because the use case consults it for the rate on an amount.
pub trait TaxRateProvider {
    fn rate_for(&self, amount: Money) -> TaxRate;
}

/// The ForCalculatingTaxes use case. It holds its driven port as a field instead of hard-coding a rate.
pub struct TaxCalculation<R> {
    rates: R,
}

impl<R: TaxRateProvider> TaxCalculation<R> {
    /// Wires the driven port into the use case.
    pub fn new(rates: R) -> Self {
        Self { rates }
    }

    /// Consults the driven port to compute the tax owed on `amount`.
    pub fn tax_on(&self, amount: Money) -> Money {
        apply_rate(amount, self.rates.rate_for(amount))
    }
}
```
