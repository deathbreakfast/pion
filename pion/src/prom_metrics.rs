//! Operational Prometheus metrics surface (via the [`metrics`] crate).
//!
//! Distinct from [`crate::logging`] (Spectra metrics and events product telemetry persisted to Valence):
//! this module emits low-cardinality counters/histograms through the global [`metrics`] recorder
//! for infra scraping (Prometheus, `pion-server`'s `GET /metrics`). Recording is a safe no-op
//! until an embedder installs a recorder, so library callers never need to check whether one is
//! present.
//!
//! Call sites live in the three control-plane entry points every embedder shares:
//! [`crate::ingest_agent_heartbeat`], [`crate::claim_pending_node_action`], and
//! [`crate::report_node_action_result`].

use std::time::Duration;

/// Records one heartbeat ingest attempt.
///
/// Emits `pion_heartbeat_ingest_total{outcome="ok"|"error"}` and
/// `pion_heartbeat_ingest_duration_seconds`.
pub fn record_heartbeat_ingest(ok: bool, elapsed: Duration) {
    let outcome = if ok { "ok" } else { "error" };
    metrics::counter!("pion_heartbeat_ingest_total", "outcome" => outcome).increment(1);
    metrics::histogram!("pion_heartbeat_ingest_duration_seconds").record(elapsed.as_secs_f64());
}

/// Records one node-action claim attempt.
///
/// Emits `pion_node_action_claim_total{result="claimed"|"no_content"|"error"}` and
/// `pion_node_action_claim_duration_seconds`.
pub fn record_node_action_claim(result: &'static str, elapsed: Duration) {
    metrics::counter!("pion_node_action_claim_total", "result" => result).increment(1);
    metrics::histogram!("pion_node_action_claim_duration_seconds").record(elapsed.as_secs_f64());
}

/// Records one node-action result report.
///
/// Emits `pion_node_action_report_total{success="true"|"false"|"error"}` (`"error"` means the
/// report call itself was rejected, e.g. unknown command or attempt mismatch — not that the
/// reported action failed) and `pion_node_action_report_duration_seconds`.
pub fn record_node_action_report(success: &'static str, elapsed: Duration) {
    metrics::counter!("pion_node_action_report_total", "success" => success).increment(1);
    metrics::histogram!("pion_node_action_report_duration_seconds").record(elapsed.as_secs_f64());
}
