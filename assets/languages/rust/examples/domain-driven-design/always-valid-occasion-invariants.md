```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Occasion {
    pub id: OccasionId,
    pub name: String,
    pub budget: Money,
    pub gift_ideas: Vec<GiftIdea>,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum InvalidOccasion {
    #[error("occasion name is required")]
    NameRequired,
    #[error("budget minor units must be non-negative")]
    NegativeBudget,
}

impl Occasion {
    /// The factory that enforces invariants on creation (the always-valid principle). It fails only for a
    /// genuine invariant violation — not for an expected business outcome. The driving edge creates the id
    /// and passes the typed value in, so the domain takes no UUID dependency.
    pub fn new(id: OccasionId, name: &str, budget: Money) -> Result<Self, InvalidOccasion> {
        let name = name.trim();
        if name.is_empty() {
            return Err(InvalidOccasion::NameRequired);
        }
        if budget.minor_units < 0 {
            return Err(InvalidOccasion::NegativeBudget);
        }
        Ok(Self { id, name: name.to_owned(), budget, gift_ideas: Vec::new() })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddGiftIdeaRejection {
    CurrencyMismatch,
    ExceedsBudget,
}

/// Enforces the budget invariant as a state transition. Expected business outcomes are returned; it panics
/// only when the aggregate's own Money invariant is already corrupt — a bug, not a business rule.
pub fn add_gift_idea(occasion: Occasion, idea: NewGiftIdea) -> Result<Occasion, AddGiftIdeaRejection> {
    let currency = occasion.budget.currency;
    if idea.estimated_cost.currency != currency || occasion.gift_ideas.iter().any(|i| i.estimated_cost.currency != currency) {
        return Err(AddGiftIdeaRejection::CurrencyMismatch);
    }
    assert!(
        idea.estimated_cost.minor_units >= 0 && occasion.gift_ideas.iter().all(|i| i.estimated_cost.minor_units >= 0),
        "invalid Money invariant"
    );
    let total_cost: i64 = occasion.gift_ideas.iter().map(|i| i.estimated_cost.minor_units).sum();
    assert!(total_cost <= occasion.budget.minor_units, "invalid Occasion budget invariant");
    if idea.estimated_cost.minor_units > occasion.budget.minor_units - total_cost {
        return Err(AddGiftIdeaRejection::ExceedsBudget);
    }
    let mut gift_ideas = occasion.gift_ideas;
    gift_ideas.push(GiftIdea { estimated_cost: idea.estimated_cost });
    Ok(Occasion { gift_ideas, ..occasion })
}
```
