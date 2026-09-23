```rust
#[derive(Debug, PartialEq)]
pub enum PledgeResult {
    Recorded(Pledged),
    Rejected(PledgeRejection),
    NotFound,
    Conflict,
}

pub trait OccasionRepository {
    async fn find_by_id(&self, id: &OccasionId) -> Result<Option<StoredOccasion>, RepositoryError>;
    async fn save_with_outbox(&self, occasion: &Occasion, events: &[PledgeRecordedEvent], expected_version: i64)
        -> Result<SaveOutcome, RepositoryError>;
}

pub trait ContributorRepository {
    async fn apply_pledge_once(&self, pledge_id: &PledgeId, contributor_id: &ContributorId, amount: Money)
        -> Result<(), RepositoryError>;
}

/// One transaction: it saves one aggregate and its outbox event together.
pub async fn handle_pledge(occasions: &impl OccasionRepository, dto: PledgeDto) -> Result<PledgeResult, RepositoryError> {
    let Some(stored) = occasions.find_by_id(&dto.occasion_id).await? else {
        return Ok(PledgeResult::NotFound);
    };
    let pledged = match record_pledge(stored.value, &dto) {
        Ok(pledged) => pledged,
        Err(reason) => return Ok(PledgeResult::Rejected(reason)),
    };
    match occasions.save_with_outbox(&pledged.occasion, &pledged.events, stored.version).await? {
        SaveOutcome::Saved => Ok(PledgeResult::Recorded(pledged)),
        SaveOutcome::Conflict => Ok(PledgeResult::Conflict),
    }
}

/// A separate, idempotent handler that converges the other aggregate.
pub async fn handle_pledge_recorded(contributors: &impl ContributorRepository, event: &PledgeRecordedEvent) -> Result<(), RepositoryError> {
    contributors.apply_pledge_once(&event.pledge_id, &event.contributor_id, event.amount).await
}
```
