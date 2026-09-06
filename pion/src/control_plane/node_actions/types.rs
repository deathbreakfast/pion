//! Wire types for the node action command queue plus the action-kind → capability mapping.

use crate::control_plane::ContainerActionKind;

/// Response returned to an agent after a successful claim.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClaimedNodeAction {
    /// Queued command row id returned to the agent.
    pub command_id: String,
    /// Action kind wire value (for example `deploy_handoff`).
    pub action_kind: String,
    /// Resolved payload (secrets substituted at claim when configured).
    pub payload_json: serde_json::Value,
    /// Attempt number for this claim (1-based after transition to `running`).
    pub attempt: i64,
    /// Orchestrator correlation key, if set at enqueue.
    pub correlation_key: Option<String>,
    /// Monotonic sequence within the correlation stream.
    pub sequence: Option<i64>,
}

/// Agent → control plane: report execution outcome for a claimed command.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReportNodeActionResult {
    /// Command id from the claim response.
    pub command_id: String,
    /// Node id that executed the action (must match the command row).
    pub node_id: String,
    /// Attempt number from the claim response.
    pub attempt: i64,
    /// True when the agent considers the action successful.
    pub success: bool,
    /// Captured stdout (truncated at persist).
    pub stdout: Option<String>,
    /// Captured stderr (truncated at persist).
    pub stderr: Option<String>,
    /// Short error summary when `success` is false.
    pub error_summary: Option<String>,
    /// Structured result payload merged into `PionNodeActionResult`.
    pub payload_json: Option<serde_json::Value>,
}

/// Queued action kind: deploy handoff runtime container (maps to deploy capability).
pub const NODE_ACTION_KIND_DEPLOY_HANDOFF: &str = "deploy_handoff";
/// Queued action kind: tear down handoff runtime container (maps to stop capability).
pub const NODE_ACTION_KIND_TEARDOWN_HANDOFF: &str = "teardown_handoff";

pub(super) fn action_kind_to_capability(kind: &str) -> Option<ContainerActionKind> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "deploy" | NODE_ACTION_KIND_DEPLOY_HANDOFF => Some(ContainerActionKind::Deploy),
        "ensure_network" => Some(ContainerActionKind::EnsureNetwork),
        "start" => Some(ContainerActionKind::Start),
        "stop" | NODE_ACTION_KIND_TEARDOWN_HANDOFF => Some(ContainerActionKind::Stop),
        "restart" => Some(ContainerActionKind::Restart),
        "logs" => Some(ContainerActionKind::Logs),
        "inspect" => Some(ContainerActionKind::Inspect),
        "probe_host" => Some(ContainerActionKind::ProbeHost),
        "diagnostic" => Some(ContainerActionKind::Diagnostic),
        "wireguard_peer" => Some(ContainerActionKind::WireguardPeer),
        "ensure_docker_image" => Some(ContainerActionKind::EnsureDockerImage),
        "grow_fs" => Some(ContainerActionKind::GrowFs),
        "templated_exec" => Some(ContainerActionKind::TemplatedExec),
        // health_check / handoff_bundle_import make agent-initiated outbound HTTP requests (SSRF
        // surface); gate them behind the same capability as other infra-provisioning actions
        // instead of letting any caller enqueue them unconditionally.
        "health_check" | "handoff_bundle_import" => Some(ContainerActionKind::Deploy),
        _ => None,
    }
}
