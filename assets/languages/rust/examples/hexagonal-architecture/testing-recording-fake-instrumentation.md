```rust
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
enum Observation {
    PledgeRejected { reason: RejectionReason, occasion_id: OccasionId },
    PledgeAccepted { amount: Money, occasion_id: OccasionId },
}

/// A recording fake for the `PledgeInstrumentation` port: each call appends an observation, so use-case tests
/// can assert on instrumentation as behaviour rather than as an afterthought.
#[derive(Default)]
struct RecordingPledgeInstrumentation {
    observed: Mutex<Vec<Observation>>,
}

impl PledgeInstrumentation for RecordingPledgeInstrumentation {
    fn pledge_rejected(&self, reason: RejectionReason, occasion_id: &OccasionId) {
        self.observed.lock().unwrap().push(Observation::PledgeRejected { reason, occasion_id: occasion_id.clone() });
    }

    fn pledge_accepted(&self, amount: Money, occasion_id: &OccasionId) {
        self.observed.lock().unwrap().push(Observation::PledgeAccepted { amount, occasion_id: occasion_id.clone() });
    }
}

#[tokio::test]
async fn announces_a_rejection_when_funding_is_closed() {
    let closed = Occasion { is_funding_closed: true, ..test_occasion() };
    let instrumentation = RecordingPledgeInstrumentation::default();
    let pledging = PledgingToOccasions::new(
        FakeOccasionRepository::with([closed.clone()]),
        FakeContributorRepository::default(),
        &instrumentation,
    );

    pledging.pledge_to_occasion(Pledge { occasion_id: closed.id.clone(), ..test_pledge() }).await.unwrap();

    let observed = instrumentation.observed.lock().unwrap();
    assert!(
        observed.contains(&Observation::PledgeRejected { reason: RejectionReason::FundingClosed, occasion_id: closed.id }),
        "{observed:?}"
    );
}
```
