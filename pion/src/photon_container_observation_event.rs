//! Photon topic for per-container state transitions observed during heartbeat ingest.

use chrono::{DateTime, Utc};

use crate::control_plane::ContainerStateTransition;
use crate::generated::PionContainerObservationState;

/// Published when ingest observes a container `state` change on a node.
#[cfg(feature = "runtime")]
#[photon::topic(name = "pion.container.state_changed", keyed_by = "node_id")]
pub struct PionContainerStateChanged {
    /// Node that reported the container.
    pub node_id: String,
    /// Cell id on the heartbeat.
    pub cell_id: String,
    /// Docker container id.
    pub container_id: String,
    /// Docker container name.
    pub container_name: String,
    /// Optional Parton instance id (handoff cell).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// Previous persisted state, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_state: Option<PionContainerObservationState>,
    /// New state after this heartbeat.
    pub next_state: PionContainerObservationState,
    /// Exit code when transitioning to exited/dead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// When the container finished, if reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<DateTime<Utc>>,
    /// Docker restart count at observation time.
    pub restart_count: u64,
    /// Agent-reported observation timestamp.
    pub observed_at: DateTime<Utc>,
}

/// Emit a state-change event for subscribers (reconcile, UI refresh).
#[cfg(feature = "runtime")]
pub async fn publish_container_state_changed(
    transition: &ContainerStateTransition,
) -> anyhow::Result<()> {
    let evt = PionContainerStateChanged {
        node_id: transition.node_id.clone(),
        cell_id: transition.cell_id.clone(),
        container_id: transition.container_id.clone(),
        container_name: transition.container_name.clone(),
        instance_id: transition.instance_id.clone(),
        prev_state: transition.prev_state.clone(),
        next_state: transition.next_state.clone(),
        exit_code: transition.exit_code,
        finished_at: transition.finished_at,
        restart_count: transition.restart_count,
        observed_at: transition.observed_at,
    };
    let _ = evt.publish().await?;
    Ok(())
}
