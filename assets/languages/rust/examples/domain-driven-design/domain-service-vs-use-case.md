```rust
/// The DOMAIN SERVICE — business logic over domain types only. Colocate it with the domain concepts it serves.
pub fn pledge_contribution(occasion: Occasion, eligibility: &ContributorEligibility, pledge: Pledge) -> PledgeDecision {
    if !eligibility.may_pledge {
        return Err(PledgeRejection::ContributorIneligible);
    }
    if occasion.is_funding_closed {
        return Err(PledgeRejection::FundingClosed);
    }
    if pledge.amount.currency != occasion.total_pledged.currency || occasion.total_pledged.currency != occasion.budget.currency {
        return Err(PledgeRejection::CurrencyMismatch);
    }
    if pledge.amount.minor_units <= 0 {
        return Err(PledgeRejection::NonPositiveAmount);
    }
    assert!(occasion.total_pledged.minor_units <= occasion.budget.minor_units, "invalid Occasion funding invariant");
    if pledge.amount.minor_units > occasion.budget.minor_units - occasion.total_pledged.minor_units {
        return Err(PledgeRejection::ExceedsBudget);
    }
    let total_pledged = Money {
        minor_units: occasion.total_pledged.minor_units.checked_add(pledge.amount.minor_units).expect("money addition overflowed"),
        ..occasion.total_pledged
    };
    let event = PledgeRecorded {
        id: pledge.id,
        occasion_id: occasion.id.clone(),
        contributor_id: eligibility.contributor_id.clone(),
        amount: pledge.amount,
    };
    Ok(Pledged { occasion: Occasion { total_pledged, ..occasion }, events: vec![event] })
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoredOccasion {
    pub value: Occasion,
    pub version: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveOutcome {
    Saved,
    Conflict,
}

/// The application-owned persistence port.
pub trait PledgePersistence {
    async fn find_occasion_by_id(&self, id: &OccasionId) -> Result<Option<StoredOccasion>, PersistenceError>;
    async fn save_with_outbox(&self, occasion: &Occasion, events: &[PledgeRecorded], expected_version: i64)
        -> Result<SaveOutcome, PersistenceError>;
}

/// The application-owned collaborator for eligibility, typically served by another bounded context.
pub trait ContributorEligibilityGateway {
    async fn find_for(&self, contributor: &ContributorId) -> Result<Option<ContributorEligibility>, PersistenceError>;
}

pub struct PledgeDto {
    pub pledge_id: PledgeId,
    pub occasion_id: OccasionId,
    pub contributor_id: ContributorId,
    pub amount: Money,
}

#[derive(Debug, PartialEq)]
pub enum PledgeResult {
    Decided(PledgeDecision),
    NotFound,
    ConcurrentChange,
}

/// The USE CASE — orchestration only, no business rules. Place it as application policy; never assume it
/// belongs in a domain module.
pub async fn handle_pledge(
    persistence: &impl PledgePersistence,
    eligibility: &impl ContributorEligibilityGateway,
    dto: PledgeDto,
) -> Result<PledgeResult, PersistenceError> {
    let stored = persistence.find_occasion_by_id(&dto.occasion_id).await?;
    let eligible = eligibility.find_for(&dto.contributor_id).await?;
    let (Some(stored), Some(eligible)) = (stored, eligible) else {
        return Ok(PledgeResult::NotFound);
    };

    let decision = pledge_contribution(stored.value, &eligible, Pledge { id: dto.pledge_id, amount: dto.amount });
    if let Ok(pledged) = &decision {
        if persistence.save_with_outbox(&pledged.occasion, &pledged.events, stored.version).await? == SaveOutcome::Conflict {
            return Ok(PledgeResult::ConcurrentChange);
        }
    }
    Ok(PledgeResult::Decided(decision))
}
```
