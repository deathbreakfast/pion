//! Node action queue metrics/events emit helpers.

use spectra_core::{try_log_event, try_record_counter};

use super::events::{node_action_log_fields, truncate_message};

/// Context for node-action telemetry (avoids long parameter lists at call sites).
#[derive(Debug, Clone)]
pub struct NodeActionContext {
    /// Queued command row id.
    pub command_id: String,
    /// Target node id.
    pub node_id: String,
    /// Action kind wire value.
    pub action_kind: String,
    /// Orchestrator correlation key (may be empty).
    pub correlation_key: String,
    /// Sequence within the correlation stream.
    pub sequence: i64,
    /// Current attempt number (0 before claim).
    pub attempt: i64,
}

impl NodeActionContext {
    /// Build context for a command before correlation fields are known.
    pub fn new(
        command_id: impl Into<String>,
        node_id: impl Into<String>,
        action_kind: impl Into<String>,
    ) -> Self {
        Self {
            command_id: command_id.into(),
            node_id: node_id.into(),
            action_kind: action_kind.into(),
            correlation_key: String::new(),
            sequence: 0,
            attempt: 0,
        }
    }

    /// Attach correlation key and sequence for structured emits.
    #[must_use]
    pub fn with_correlation(mut self, correlation_key: impl Into<String>, sequence: i64) -> Self {
        self.correlation_key = correlation_key.into();
        self.sequence = sequence;
        self
    }

    /// Attach attempt number after claim or result reporting.
    #[must_use]
    pub fn with_attempt(mut self, attempt: i64) -> Self {
        self.attempt = attempt;
        self
    }

    /// Emit metric counter + event for a queue lifecycle step.
    pub fn log_step(&self, action: &str, outcome: &str, message: &str) {
        try_record_counter(
            "pion_node_actions",
            &[("action", action), ("outcome", outcome)],
            1,
        );
        try_log_event(
            "pion_node_action_log",
            &node_action_log_fields(
                &self.command_id,
                &self.node_id,
                &self.action_kind,
                &self.correlation_key,
                self.sequence,
                self.attempt,
                outcome,
                message,
            ),
        );
    }

    /// Emit enqueue lifecycle telemetry.
    pub fn log_enqueue(&self, outcome: &str, message: &str) {
        self.log_step("enqueue", outcome, message);
    }

    /// Emit successful claim telemetry.
    pub fn log_claim(&self, message: &str) {
        self.log_step("claim", "claimed", message);
    }

    /// Emit result reporting telemetry (`success`, `retry`, `failed`, …).
    pub fn log_result(&self, outcome: &str, message: &str) {
        self.log_step("result", outcome, message);
    }

    /// Emit lease extension telemetry.
    pub fn log_lease_extend(&self, message: &str) {
        self.log_step("lease_extend", "ok", message);
    }

    /// Emit reconcile recovery telemetry (pending window extended).
    pub fn log_recovery(&self, message: &str) {
        self.log_step("recovery", "extended_pending", message);
    }
}

/// Format a no-op enqueue message for structured emit.
pub fn enqueue_noop_message(status: &str, missing_replan: bool) -> String {
    if missing_replan {
        format!("enqueue no-op (terminal {status}, missing replan_token)")
    } else {
        format!("enqueue no-op (already {status})")
    }
}

/// Format a terminal failure message including stderr length.
pub fn terminal_failure_message(last_error: &str, stderr_len: usize) -> String {
    truncate_message(&format!(
        "result failed_terminal last_error={last_error} stderr_len={stderr_len}"
    ))
}
