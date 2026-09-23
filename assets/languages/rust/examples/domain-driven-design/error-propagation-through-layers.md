```rust
/// The domain function: its result carries the business reason.
pub fn pledge_contribution(occasion: Occasion, eligibility: &Eligibility, pledge: NewPledge) -> PledgeDecision {
    if occasion.is_funding_closed {
        return Err(PledgeRejection::FundingClosed);
    }
    // ...
}

#[derive(Debug, PartialEq)]
pub enum PledgeResult {
    Decided(PledgeDecision),
    NotFound,
    ConcurrentChange,
}

/// The use case: it propagates the domain result and adds the application-level outcomes while orchestrating
/// persistence. `?` carries infrastructure failure up unchanged.
pub async fn handle_pledge(repos: &Repos, dto: PledgeDto) -> Result<PledgeResult, RepositoryError> {
    let stored = repos.occasions.find_by_id(&dto.occasion_id).await?;
    let eligibility = repos.eligibility.find_for(&dto.contributor_id).await?;
    let (Some(stored), Some(eligibility)) = (stored, eligibility) else {
        return Ok(PledgeResult::NotFound);
    };

    let decision = pledge_contribution(stored.value, &eligibility, NewPledge { id: dto.pledge_id, amount: dto.amount });
    if let Ok(pledged) = &decision {
        if repos.occasions.save_with_outbox(&pledged.occasion, &pledged.events, stored.version).await? == SaveOutcome::Conflict {
            return Ok(PledgeResult::ConcurrentChange);
        }
    }
    Ok(PledgeResult::Decided(decision))
}

/// The delivery-layer translator: result to HTTP status, exhaustively.
pub fn to_http_status(result: &PledgeResult) -> StatusCode {
    match result {
        PledgeResult::NotFound => StatusCode::NOT_FOUND,
        PledgeResult::ConcurrentChange => StatusCode::CONFLICT,
        PledgeResult::Decided(Ok(_)) => StatusCode::OK,
        PledgeResult::Decided(Err(_)) => StatusCode::UNPROCESSABLE_ENTITY,
    }
}
```
