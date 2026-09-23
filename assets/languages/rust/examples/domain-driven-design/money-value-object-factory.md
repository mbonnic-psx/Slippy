```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    Gbp,
    Usd,
    Eur,
}

/// An invariant violation — a bug in calling code, not a user-input problem.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum InvalidMoney {
    #[error("money minor units overflow the safe integer range")]
    Overflow,
    #[error("money cannot be negative")]
    Negative,
}

/// JavaScript's `Number.MAX_SAFE_INTEGER`, so the same class of overflow is refused explicitly wherever these
/// values cross into a JSON client, rather than trusting the much wider range of `i64`.
const MAX_SAFE_INTEGER: i64 = (1 << 53) - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money {
    minor_units: i64,
    currency: Currency,
}

impl Money {
    /// Validates and constructs. Untrusted input is validated at the trust boundary before this is called; an
    /// `Err` here means an invariant was broken (a bug), not that a user typed something wrong.
    pub fn new(minor_units: i64, currency: Currency) -> Result<Self, InvalidMoney> {
        if minor_units.abs() > MAX_SAFE_INTEGER {
            return Err(InvalidMoney::Overflow);
        }
        if minor_units < 0 {
            return Err(InvalidMoney::Negative);
        }
        Ok(Self { minor_units, currency })
    }

    pub fn minor_units(self) -> i64 {
        self.minor_units
    }

    pub fn currency(self) -> Currency {
        self.currency
    }
}
```
