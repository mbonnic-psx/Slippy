```rust
use std::cell::RefCell;
use std::collections::HashMap;

// --- in-memory fakes for the use-case test ---

#[derive(Default)]
struct FakePledgePersistence {
    occasions: RefCell<HashMap<OccasionId, Occasion>>,
    saved: RefCell<Vec<Occasion>>,
    outbox: RefCell<Vec<PledgeRecorded>>,
}

impl PledgePersistence for FakePledgePersistence {
    async fn find_occasion_by_id(&self, id: &OccasionId) -> Result<Option<StoredOccasion>, PersistenceError> {
        Ok(self.occasions.borrow().get(id).map(|occasion| StoredOccasion { value: occasion.clone(), version: 1 }))
    }

    async fn save_with_outbox(&self, occasion: &Occasion, events: &[PledgeRecorded], _expected_version: i64)
        -> Result<SaveOutcome, PersistenceError>
    {
        self.occasions.borrow_mut().insert(occasion.id.clone(), occasion.clone());
        self.saved.borrow_mut().push(occasion.clone());
        self.outbox.borrow_mut().extend_from_slice(events);
        Ok(SaveOutcome::Saved)
    }
}

#[derive(Default)]
struct FakeEligibilityGateway {
    entries: HashMap<ContributorId, ContributorEligibility>,
}

impl ContributorEligibilityGateway for FakeEligibilityGateway {
    async fn find_for(&self, contributor: &ContributorId) -> Result<Option<ContributorEligibility>, PersistenceError> {
        Ok(self.entries.get(contributor).cloned())
    }
}

#[tokio::test]
async fn refuses_an_ineligible_contributor_and_saves_nothing() {
    let occasion = Occasion {
        id: OccasionId("occasion-1".into()),
        is_funding_closed: false,
        total_pledged: Money { minor_units: 0, currency: Currency::Gbp },
        budget: Money { minor_units: 50_000, currency: Currency::Gbp },
    };
    let contributor = ContributorId("contributor-1".into());
    let persistence = FakePledgePersistence::default();
    persistence.occasions.borrow_mut().insert(occasion.id.clone(), occasion.clone());
    let gateway = FakeEligibilityGateway {
        entries: [(contributor.clone(), ContributorEligibility { contributor_id: contributor.clone(), may_pledge: false })].into(),
    };

    let result = handle_pledge(&persistence, &gateway, PledgeDto {
        pledge_id: PledgeId("pledge-1".into()),
        occasion_id: occasion.id,
        contributor_id: contributor,
        amount: Money { minor_units: 5_000, currency: Currency::Gbp },
    })
    .await
    .expect("no infrastructure failure");

    assert_eq!(result, PledgeResult::Decided(Err(PledgeRejection::ContributorIneligible)));
    assert!(persistence.saved.borrow().is_empty());
    assert!(persistence.outbox.borrow().is_empty());
}
```
