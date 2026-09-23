```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Occasion {
    pub id: OccasionId,
    pub name: String,
    pub gift_ideas: Vec<GiftIdea>,
    pub budget: Money,
    pub total_pledged: Money,
    pub is_funding_closed: bool,
}

/// A new `Occasion` with the name changed. It takes the old one by value, so there is no original left behind
/// to be half-updated: struct-update syntax moves every other field across unchanged.
pub fn rename_occasion(occasion: Occasion, new_name: String) -> Occasion {
    Occasion { name: new_name, ..occasion }
}
```
