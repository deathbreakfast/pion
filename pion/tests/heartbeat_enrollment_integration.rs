//! End-to-end heartbeat ingest on in-memory Valence (enrollment + observed status).
//!
//! Complements the `parton` crate HTTP integration tests by exercising
//! [`pion::ingest_agent_heartbeat`] directly against the same logical-router shape a host
//! server uses.

#![cfg(feature = "runtime")]
// Integration tests use `.expect()` / `.unwrap()` for brevity on fixtures.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use chrono::Utc;
use pion::generated::{
    PionAgentHostEnrollment, PionAgentHostEnrollmentStatus, PionControlPlaneObservedStatus,
};
use pion::{
    create_host_enrollment, ingest_agent_heartbeat, ContainerStatusSummary, HostCapabilities,
    NodeHeartbeatReport,
};
use valence::Model;

use common::containers_from_summary;
#[tokio::test]
async fn ingest_heartbeat_creates_observed_status_snapshot() -> anyhow::Result<()> {
    let v = common::test_valence("pion_heartbeat_enrollment").await;

    let (_, token) = create_host_enrollment(
        &v,
        "127.0.0.1".to_string(),
        "local-default".to_string(),
        "default".to_string(),
        86_400,
    )
    .await?;

    let report = NodeHeartbeatReport {
        node_id: "agent-node-pion-it".to_string(),
        cell_id: "local-default".to_string(),
        capabilities: HostCapabilities {
            hostname: "agent-node-pion-it.local".to_string(),
            arch: "x86_64".to_string(),
            cpu_logical: 4,
            memory_bytes: 8 * 1024 * 1024 * 1024,
            mounts: vec![],
            labels: serde_json::json!({}),
        },
        containers: containers_from_summary(ContainerStatusSummary {
            running: 1,
            exited: 0,
            unhealthy: 0,
        }),
        observed_at: Utc::now(),
        enrollment_token: Some(token),
        applied_directive_token: None,
        apply_failed: None,
    };

    ingest_agent_heartbeat(&report, Some("127.0.0.1"), &v).await?;

    let observed = PionControlPlaneObservedStatus::query(&v, valence::use_!("**Test:** Fixture **Pion Control Plane Observed Status** list for `tests` so the suite can arrange and assert persistence behavior. CI and developers running the suite only.")).await?;
    assert_eq!(observed.len(), 1, "expected one observed status snapshot");
    assert_eq!(observed[0].source().as_str(), "agent");
    Ok(())
}

fn first_heartbeat(node_id: &str, cell_id: &str, token: String) -> NodeHeartbeatReport {
    NodeHeartbeatReport {
        node_id: node_id.to_string(),
        cell_id: cell_id.to_string(),
        capabilities: HostCapabilities {
            hostname: format!("{node_id}.local"),
            arch: "x86_64".to_string(),
            cpu_logical: 4,
            memory_bytes: 8 * 1024 * 1024 * 1024,
            mounts: vec![],
            labels: serde_json::json!({}),
        },
        containers: containers_from_summary(ContainerStatusSummary {
            running: 1,
            exited: 0,
            unhealthy: 0,
        }),
        observed_at: Utc::now(),
        enrollment_token: Some(token),
        applied_directive_token: None,
        apply_failed: None,
    }
}

async fn load_enrollment(v: &valence::Valence, id: &str) -> PionAgentHostEnrollment {
    PionAgentHostEnrollment::get(id, v, valence::use_!("**Test:** Fixture **Pion Agent Host Enrollment** read for `tests` so the suite can assert the recorded enrollment failure. CI and developers running the suite only."))
        .await
        .unwrap()
        .expect("enrollment row")
}

#[tokio::test]
async fn rejected_token_secret_is_recorded_on_pending_ticket() -> anyhow::Result<()> {
    let v = common::test_valence("pion_heartbeat_enrollment_bad_secret").await;
    let (id, token) = create_host_enrollment(
        &v,
        "127.0.0.1".to_string(),
        "local-default".to_string(),
        "default".to_string(),
        86_400,
    )
    .await?;
    let wrong = format!("{}.wrongsecret", token.rsplit_once('.').unwrap().0);
    assert_ne!(wrong, token);

    let report = first_heartbeat("agent-node-bad-secret", "local-default", wrong);
    assert!(ingest_agent_heartbeat(&report, Some("127.0.0.1"), &v)
        .await
        .is_err());

    let row = load_enrollment(&v, &id).await;
    assert_eq!(*row.status(), PionAgentHostEnrollmentStatus::Pending);
    assert_eq!(
        row.last_error().map(String::as_str),
        Some("enrollment token mismatch")
    );
    assert!(row.last_error_at().is_some());
    Ok(())
}

#[tokio::test]
async fn rejected_cell_is_recorded_on_pending_ticket() -> anyhow::Result<()> {
    let v = common::test_valence("pion_heartbeat_enrollment_bad_cell").await;
    let (id, token) = create_host_enrollment(
        &v,
        "127.0.0.1".to_string(),
        "local-default".to_string(),
        "default".to_string(),
        86_400,
    )
    .await?;

    let report = first_heartbeat("agent-node-bad-cell", "some-other-cell", token);
    assert!(ingest_agent_heartbeat(&report, Some("127.0.0.1"), &v)
        .await
        .is_err());

    let row = load_enrollment(&v, &id).await;
    assert_eq!(*row.status(), PionAgentHostEnrollmentStatus::Pending);
    assert_eq!(
        row.last_error().map(String::as_str),
        Some("heartbeat cell_id does not match enrollment")
    );
    Ok(())
}

#[tokio::test]
async fn claimed_ticket_has_no_last_error() -> anyhow::Result<()> {
    let v = common::test_valence("pion_heartbeat_enrollment_claimed_ok").await;
    let (id, token) = create_host_enrollment(
        &v,
        "127.0.0.1".to_string(),
        "local-default".to_string(),
        "default".to_string(),
        86_400,
    )
    .await?;

    let report = first_heartbeat("agent-node-claimed-ok", "local-default", token);
    ingest_agent_heartbeat(&report, Some("127.0.0.1"), &v).await?;

    let row = load_enrollment(&v, &id).await;
    assert_eq!(*row.status(), PionAgentHostEnrollmentStatus::Claimed);
    assert!(row.last_error().is_none());
    assert!(row.last_error_at().is_none());
    Ok(())
}
