```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OccasionId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct Occasion {
    pub id: OccasionId,
    pub name: String,
}

/// An application-owned repository contract — an inside-owned port when hexagonal architecture is used.
/// `Option` says whether an occasion with that id exists, separately from any infrastructure `Err`.
pub trait OccasionRepository {
    async fn find_by_id(&self, id: &OccasionId) -> Result<Option<Occasion>, RepositoryError>;
    async fn save(&self, occasion: &Occasion) -> Result<(), RepositoryError>;
}

// Concrete implementations belong with infrastructure; in hexagonal architecture they are driven adapters.
```
