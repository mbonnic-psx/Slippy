//! One contract, run against every checkpoint store — the read side's twin of the event-store contract.
//!
//! The case that matters most is the unit-of-work one: a checkpoint recorded in a unit of work that fails
//! must not move, and the append beside it must roll back with it. That is the whole difference between a
//! checkpoint and a race with a number in it.

use std::future::Future;
use std::time::Duration;

use time::OffsetDateTime;
use time::macros::datetime;
use uuid::Uuid;

use crate::application::ports::events::{
    Actor, CorrelationId, DomainEvent, EventStore, NO_STREAM, StoreError,
};
use crate::application::ports::read_models::{CheckpointStore, FROM_THE_BEGINNING, Lease};

/// The instant every lease below is claimed at: a test that cannot choose the time cannot test expiry
/// without sleeping.
pub const NOW: OffsetDateTime = datetime!(2026-09-10 12:00 UTC);
const A_WHILE: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
#[error("the view write failed")]
struct ViewFailed;

fn projection() -> String {
    format!("contract-{}", Uuid::new_v4().simple())
}

async fn position(checkpoints: &impl CheckpointStore, projection: &str) -> i64 {
    checkpoints
        .position_of(projection)
        .await
        .expect("read a position")
}

async fn claim(
    checkpoints: &impl CheckpointStore,
    projection: &str,
    owner: &str,
    now: OffsetDateTime,
) -> Option<Lease> {
    checkpoints
        .claim(projection, owner, now, A_WHILE)
        .await
        .expect("claim")
}

/// Holds a checkpoint store to the port's contract. `new_stores` builds an event store and the checkpoint
/// store that follows it, fresh for each case.
pub async fn run<E, C, Fut>(new_stores: impl Fn() -> Fut)
where
    E: EventStore,
    C: CheckpointStore,
    Fut: Future<Output = (E, C)>,
{
    // starts a projection nothing has recorded at the beginning
    {
        let (_, checkpoints) = new_stores().await;
        assert_eq!(
            position(&checkpoints, &projection()).await,
            FROM_THE_BEGINNING
        );
    }

    // records a position and reads it back
    {
        let (_, checkpoints) = new_stores().await;
        let name = projection();
        checkpoints.record(&name, 42).await.expect("record");
        assert_eq!(position(&checkpoints, &name).await, 42);
    }

    // records the same projection twice without a second row
    {
        let (_, checkpoints) = new_stores().await;
        let name = projection();
        checkpoints.record(&name, 7).await.expect("record");
        checkpoints.record(&name, 9).await.expect("record");
        assert_eq!(
            position(&checkpoints, &name).await,
            9,
            "the second write should win"
        );
    }

    // puts a position back to the beginning for a rebuild
    {
        let (_, checkpoints) = new_stores().await;
        let name = projection();
        checkpoints.record(&name, 99).await.expect("record");
        checkpoints
            .record(&name, FROM_THE_BEGINNING)
            .await
            .expect("record");
        assert_eq!(position(&checkpoints, &name).await, FROM_THE_BEGINNING);
    }

    // does not record a position written in a unit of work that fails
    {
        let (store, checkpoints) = new_stores().await;
        let name = projection();
        let stream = format!("checkpoint-{}", Uuid::new_v4().simple());
        let event = DomainEvent {
            event_type: "Started".to_owned(),
            schema_version: 1,
            stream_id: stream.clone(),
            payload: serde_json::Map::new(),
            occurred_at: "2024-01-01T00:00:00.000000+00:00".to_owned(),
            actor: Actor {
                kind: "test".to_owned(),
                id: "checkpoints".to_owned(),
            },
            correlation_id: CorrelationId::generate(),
            causation_id: None,
        };
        let outcome: Result<(), StoreError> = store
            .in_unit_of_work(|| async {
                store.append(&stream, NO_STREAM, vec![event]).await?;
                checkpoints.record(&name, 5).await?;
                Err(StoreError::new("the view", ViewFailed))
            })
            .await;
        assert!(
            outcome.is_err(),
            "the view's failure should reach the caller"
        );
        assert_eq!(
            position(&checkpoints, &name).await,
            FROM_THE_BEGINNING,
            "the checkpoint moved"
        );
        assert!(
            store.read(&stream).await.expect("read").is_empty(),
            "the append did not roll back with it"
        );
    }

    // claims a projection for one owner
    {
        let (_, checkpoints) = new_stores().await;
        let lease = claim(&checkpoints, &projection(), "worker-1", NOW)
            .await
            .expect("a lease");
        assert_eq!(
            (lease.owner.as_str(), lease.expires_at),
            ("worker-1", NOW + A_WHILE)
        );
    }

    // refuses a claim while somebody else holds an unexpired lease
    {
        let (_, checkpoints) = new_stores().await;
        let name = projection();
        claim(&checkpoints, &name, "worker-1", NOW).await;
        assert!(
            claim(
                &checkpoints,
                &name,
                "worker-2",
                NOW + Duration::from_secs(1)
            )
            .await
            .is_none()
        );
    }

    // lets the owner renew its own lease
    {
        let (_, checkpoints) = new_stores().await;
        let name = projection();
        claim(&checkpoints, &name, "worker-1", NOW).await;
        let later = NOW + Duration::from_secs(10);
        let renewed = claim(&checkpoints, &name, "worker-1", later)
            .await
            .expect("renewed");
        assert_eq!(renewed.expires_at, later + A_WHILE);
    }

    // lets another owner claim a lapsed lease
    {
        let (_, checkpoints) = new_stores().await;
        let name = projection();
        claim(&checkpoints, &name, "worker-1", NOW).await;
        let taken = claim(
            &checkpoints,
            &name,
            "worker-2",
            NOW + A_WHILE + Duration::from_secs(1),
        )
        .await;
        assert_eq!(taken.map(|lease| lease.owner), Some("worker-2".to_owned()));
    }

    // lets another owner claim a lease at the very instant it expires
    {
        let (_, checkpoints) = new_stores().await;
        let name = projection();
        claim(&checkpoints, &name, "worker-1", NOW).await;
        let taken = claim(&checkpoints, &name, "worker-2", NOW + A_WHILE).await;
        assert_eq!(taken.map(|lease| lease.owner), Some("worker-2".to_owned()));
    }

    // makes a released lease claimable at once
    {
        let (_, checkpoints) = new_stores().await;
        let name = projection();
        let lease = claim(&checkpoints, &name, "worker-1", NOW)
            .await
            .expect("the first claim");
        checkpoints.release(&lease).await.expect("release");
        assert!(claim(&checkpoints, &name, "worker-2", NOW).await.is_some());
    }

    // does nothing when releasing a lease somebody else holds
    {
        let (_, checkpoints) = new_stores().await;
        let name = projection();
        claim(&checkpoints, &name, "worker-1", NOW).await;
        let not_theirs = Lease {
            projection: name.clone(),
            owner: "worker-2".to_owned(),
            expires_at: NOW,
        };
        checkpoints.release(&not_theirs).await.expect("release");
        assert!(
            claim(&checkpoints, &name, "worker-3", NOW).await.is_none(),
            "worker-1's lease was released"
        );
    }

    // does not disturb the position when the lease moves
    {
        let (_, checkpoints) = new_stores().await;
        let name = projection();
        checkpoints.record(&name, 12).await.expect("record");
        let lease = claim(&checkpoints, &name, "worker-1", NOW)
            .await
            .expect("claim");
        checkpoints.release(&lease).await.expect("release");
        assert_eq!(position(&checkpoints, &name).await, 12);
    }
}
