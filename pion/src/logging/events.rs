//! Event field builders for Pion product telemetry.

use serde_json::{json, Value};

const MAX_MESSAGE_LEN: usize = 512;

pub fn truncate_message(message: &str) -> String {
    if message.len() <= MAX_MESSAGE_LEN {
        message.to_string()
    } else {
        format!("{}…", &message[..MAX_MESSAGE_LEN.saturating_sub(1)])
    }
}

pub fn heartbeat_log_fields(node_id: &str, cell_id: &str, state: &str, message: &str) -> Value {
    json!({
        "node_id": node_id,
        "cell_id": cell_id,
        "state": state,
        "message": truncate_message(message),
    })
}

#[allow(clippy::too_many_arguments)] // Event node-action log field builder mirrors queue row shape
pub fn node_action_log_fields(
    command_id: &str,
    node_id: &str,
    action_kind: &str,
    correlation_key: &str,
    sequence: i64,
    attempt: i64,
    outcome: &str,
    message: &str,
) -> Value {
    json!({
        "command_id": command_id,
        "node_id": node_id,
        "action_kind": action_kind,
        "correlation_key": correlation_key,
        "sequence": sequence,
        "attempt": attempt,
        "outcome": outcome,
        "message": truncate_message(message),
    })
}

pub fn handoff_log_fields(
    directive_id: &str,
    node_id: &str,
    phase: &str,
    reason: &str,
    message: &str,
) -> Value {
    json!({
        "directive_id": directive_id,
        "node_id": node_id,
        "phase": phase,
        "reason": reason,
        "message": truncate_message(message),
    })
}

pub fn migration_log_fields(
    operation: &str,
    from_table: &str,
    to_table: &str,
    rows_copied: i64,
    outcome: &str,
    message: &str,
) -> Value {
    json!({
        "operation": operation,
        "from_table": from_table,
        "to_table": to_table,
        "rows_copied": rows_copied,
        "outcome": outcome,
        "message": truncate_message(message),
    })
}

pub fn server_boot_log_fields(step: &str, message: &str) -> Value {
    json!({
        "step": step,
        "message": truncate_message(message),
    })
}
