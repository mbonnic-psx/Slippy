```rust
/// Distinct newtypes, so ids of different entities cannot be mixed up at compile time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OccasionId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GiftIdeaId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    Gbp,
    Usd,
    Eur,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money {
    pub minor_units: i64,
    pub currency: Currency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GiftIdeaStatus {
    Proposed,
    Selected,
    Purchased,
}

/// Domain language.
pub struct GiftIdea {
    pub id: GiftIdeaId,
    pub description: String,
    pub occasion: OccasionId,
    pub estimated_cost: Money,
    pub status: GiftIdeaStatus,
}

/// Technical jargon — avoid this shape. `id`/`text`/`parent_id` say nothing about the domain, unlike
/// `GiftIdea`'s fields above.
pub struct Item {
    pub id: String,
    pub text: String,
    pub parent_id: String,
}
```
