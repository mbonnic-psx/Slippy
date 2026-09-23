```rust
pub struct Occasion {
    pub id: OccasionId,
    pub version: i64, // incremented on each save
    pub name: String,
    pub budget: Money,
    pub gift_ideas: Vec<GiftIdea>,
}
```
