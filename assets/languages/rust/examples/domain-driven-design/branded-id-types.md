```rust
/// Distinct newtypes, so an id belonging to one entity can never be passed where the other is expected,
/// though both wrap a `String`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OccasionId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GiftIdeaId(String);

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("{0} id cannot be empty")]
pub struct EmptyId(&'static str);

impl OccasionId {
    pub fn new(raw: &str) -> Result<Self, EmptyId> {
        if raw.trim().is_empty() {
            return Err(EmptyId("occasion"));
        }
        Ok(Self(raw.to_owned()))
    }
}

impl GiftIdeaId {
    pub fn new(raw: &str) -> Result<Self, EmptyId> {
        if raw.trim().is_empty() {
            return Err(EmptyId("gift idea"));
        }
        Ok(Self(raw.to_owned()))
    }
}
```
