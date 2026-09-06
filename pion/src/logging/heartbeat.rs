//! Heartbeat ingest metrics/events emit helpers.

use spectra_core::{try_log_event, try_record_counter};

use super::events::heartbeat_log_fields;

/// Context for heartbeat telemetry.
#[derive(Debug, Clone)]
pub struct HeartbeatContext {
    /// Node id from the heartbeat report.
    pub node_id: String,
    /// Cell id from the heartbeat report.
    pub cell_id: String,
}

impl HeartbeatContext {
    /// Build context from ingest report ids.
    pub fn new(node_id: impl Into<String>, cell_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            cell_id: cell_id.into(),
        }
    }

    /// Record successful heartbeat ingest.
    pub fn record_ingested_ok(&self) {
        try_record_counter("pion_heartbeats_ingested", &[("outcome", "ok")], 1);
    }

    /// Record failed heartbeat ingest.
    pub fn record_ingested_error(&self, message: &str) {
        try_record_counter("pion_heartbeats_ingested", &[("outcome", "error")], 1);
        try_log_event(
            "pion_heartbeat_log",
            &heartbeat_log_fields(&self.node_id, &self.cell_id, "error", message),
        );
    }

    /// Record container state Photon publish outcome.
    pub fn record_publish(&self, outcome: &str, message: &str) {
        try_record_counter("pion_heartbeat_publishes", &[("outcome", outcome)], 1);
        if outcome != "success" {
            try_log_event(
                "pion_heartbeat_log",
                &heartbeat_log_fields(&self.node_id, &self.cell_id, "publish", message),
            );
        }
    }
}
