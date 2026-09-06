//! Queued node action commands: enqueue (control plane), claim/result (agent), reconciliation.
//!
//! Node action command queue.

mod claim;
mod enqueue;
mod error;
mod lifecycle;
mod report;
mod shared;
mod types;

pub use claim::claim_pending_node_action;
pub use enqueue::{enqueue_node_action, enqueue_node_action_idempotent};
pub use error::NodeActionError;
pub use lifecycle::{
    cancel_node_action, cancel_non_terminal_node_actions_for_correlation,
    default_lease_duration_secs, default_max_attempts, extend_node_action_lease,
    get_node_action_command, pending_timeout_duration, reconcile_node_action_commands,
    reset_failed_node_actions_for_correlation,
};
pub use report::report_node_action_result;
pub use types::{
    ClaimedNodeAction, ReportNodeActionResult, NODE_ACTION_KIND_DEPLOY_HANDOFF,
    NODE_ACTION_KIND_TEARDOWN_HANDOFF,
};
