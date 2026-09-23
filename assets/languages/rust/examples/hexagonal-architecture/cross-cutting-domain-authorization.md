```rust
/// Why funding cannot be closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseFundingRejection {
    NotOrganizer,
}

/// Domain logic: "only the organiser can close funding" is a business rule, returned as a value — never a
/// panic.
pub fn close_funding(occasion: Occasion, requester: &ContributorId) -> Result<Occasion, CloseFundingRejection> {
    if occasion.organizer_id != *requester {
        return Err(CloseFundingRejection::NotOrganizer);
    }
    Ok(Occasion { is_funding_closed: true, ..occasion })
}
```
