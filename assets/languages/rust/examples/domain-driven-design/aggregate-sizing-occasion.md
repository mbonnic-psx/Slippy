```rust
// ❌ WRONG: too large — users do not belong in the aggregate.
pub struct Occasion {
    pub organizer: User,         // embedded user — wrong!
    pub contributors: Vec<User>, // embedded users — wrong!
    pub gift_ideas: Vec<GiftIdea>,
}

// ✅ RIGHT: right-sized — only what consistency needs.
pub struct Occasion {
    pub organizer_id: UserId,      // reference by id
    pub gift_ideas: Vec<GiftIdea>, // owned — needed for the budget invariant
    pub budget: Money,             // owned — needed for the budget invariant
}
```
