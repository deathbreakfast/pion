//! Handoff directive metrics/events emit helpers.

use spectra_core::{try_log_event, try_record_counter};

use super::events::handoff_log_fields;

/// Context for handoff directive telemetry.
#[derive(Debug, Clone, Default)]
pub struct HandoffDirectiveContext {
    /// Handoff directive row id (may be empty before upsert).
    pub directive_id: String,
    /// Target node id for the directive.
    pub node_id: String,
}

impl HandoffDirectiveContext {
    /// Build context with directive and node ids.
    pub fn new(directive_id: impl Into<String>, node_id: impl Into<String>) -> Self {
        Self {
            directive_id: directive_id.into(),
            node_id: node_id.into(),
        }
    }

    /// Build context when only the node id is known (collect phase).
    pub fn from_node(node_id: impl Into<String>) -> Self {
        Self {
            directive_id: String::new(),
            node_id: node_id.into(),
        }
    }

    /// Emit metrics + events for a handoff phase outcome.
    pub fn log_phase(&self, phase: &str, outcome: &str, reason: &str, message: &str) {
        try_record_counter(
            "pion_handoff_directives",
            &[("phase", phase), ("outcome", outcome)],
            1,
        );
        try_log_event(
            "pion_handoff_log",
            &handoff_log_fields(&self.directive_id, &self.node_id, phase, reason, message),
        );
    }

    /// Log deliver-phase failure when state transition fails.
    pub fn warn_deliver_failed(&self, directive_id: &str, error: &str) {
        let ctx = Self::new(directive_id, &self.node_id);
        ctx.log_phase("deliver", "error", "transition_failed", error);
    }

    /// Log collect skip when a directive row has no id.
    pub fn warn_missing_id(&self) {
        self.log_phase(
            "collect",
            "skipped",
            "missing_id",
            "handoff directive row missing id; skipping",
        );
    }

    /// Log ack skip when acknowledgment persistence fails.
    pub fn warn_ack_skipped(&self, error: &str) {
        self.log_phase("ack", "skipped", "ack_failed", error);
    }

    /// Log collect skip when directive collection fails.
    pub fn warn_collect_skipped(&self, error: &str) {
        self.log_phase("collect", "skipped", "collect_failed", error);
    }
}
