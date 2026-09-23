```rust
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    Gbp,
    Usd,
    Eur,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OccasionId(String);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContributorId(String);

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum InvalidInput {
    #[error("{0} id cannot be empty")]
    EmptyId(&'static str),
    #[error("amount.minor_units must be a positive integer")]
    NonPositiveAmount,
    #[error("unsupported currency: {0}")]
    UnsupportedCurrency(String),
}

impl OccasionId {
    pub fn new(raw: &str) -> Result<Self, InvalidInput> {
        if raw.trim().is_empty() {
            return Err(InvalidInput::EmptyId("occasion"));
        }
        Ok(Self(raw.to_owned()))
    }
}

impl ContributorId {
    pub fn new(raw: &str) -> Result<Self, InvalidInput> {
        if raw.trim().is_empty() {
            return Err(InvalidInput::EmptyId("contributor"));
        }
        Ok(Self(raw.to_owned()))
    }
}

/// The untrusted shape as received — decoded JSON, every field still just data.
#[derive(Debug, Deserialize)]
pub struct RawPledgeInput {
    pub occasion_id: String,
    pub contributor_id: String,
    pub amount: RawAmount,
}

#[derive(Debug, Deserialize)]
pub struct RawAmount {
    pub minor_units: i64,
    pub currency: String,
}

/// The validated shape produced at the trust boundary.
#[derive(Debug, PartialEq)]
pub struct PledgeInput {
    pub occasion_id: OccasionId,
    pub contributor_id: ContributorId,
    pub amount: Money,
}

fn parse_currency(raw: &str) -> Result<Currency, InvalidInput> {
    serde_json::from_value(serde_json::Value::String(raw.to_owned()))
        .map_err(|_| InvalidInput::UnsupportedCurrency(raw.to_owned()))
}

/// Parses raw, untrusted input into validated domain types at the trust boundary. An `Err` means the caller
/// (an HTTP handler, say) answers with a 4xx, not a 500.
impl TryFrom<RawPledgeInput> for PledgeInput {
    type Error = InvalidInput;

    fn try_from(raw: RawPledgeInput) -> Result<Self, InvalidInput> {
        if raw.amount.minor_units <= 0 {
            return Err(InvalidInput::NonPositiveAmount);
        }
        Ok(PledgeInput {
            occasion_id: OccasionId::new(&raw.occasion_id)?,
            contributor_id: ContributorId::new(&raw.contributor_id)?,
            amount: Money { minor_units: raw.amount.minor_units, currency: parse_currency(&raw.amount.currency)? },
        })
    }
}

/// The raw persistence shape as read from storage.
#[derive(Debug, sqlx::FromRow)]
pub struct OccasionRow {
    pub id: String,
    pub name: String,
    pub budget_minor_units: i64,
    pub budget_currency: String,
    pub pledged_minor_units: i64,
    pub is_funding_closed: bool,
}

/// Reconstitutes an `Occasion` from persistence through the same validating constructors the trust-boundary
/// parser uses. It runs at the integration boundary — a driven adapter loads the gift ideas beside the row.
pub fn occasion_from_row(row: OccasionRow, gift_ideas: Vec<GiftIdea>) -> Result<Occasion, InvalidInput> {
    let currency = parse_currency(&row.budget_currency)?;
    Ok(Occasion {
        id: OccasionId::new(&row.id)?,
        name: row.name,
        gift_ideas,
        budget: Money { minor_units: row.budget_minor_units, currency },
        total_pledged: Money { minor_units: row.pledged_minor_units, currency },
        is_funding_closed: row.is_funding_closed,
    })
}
```
