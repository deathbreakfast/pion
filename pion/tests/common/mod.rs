//! Shared Valence router setup for Pion integration tests (`SQLite` `:memory:`).

// Shared test helpers use `.expect()` / `.unwrap()` for brevity on fixtures.
#![allow(clippy::unwrap_used, clippy::expect_used)]
use parton::{ContainerStatusReport, ContainerStatusSummary};
use valence::Valence;

/// Legacy heartbeat tests that only need roll-up counts.
#[allow(dead_code)]
pub fn containers_from_summary(summary: ContainerStatusSummary) -> ContainerStatusReport {
    ContainerStatusReport {
        containers: vec![],
        summary,
    }
}

/// Builds an in-memory `SQLite` [`Valence`] suitable for control-plane integration tests.
pub async fn test_valence(operation: &str) -> Valence {
    let boot = pion::valence_bootstrap::bootstrap_sqlite_memory()
        .await
        .expect("sqlite memory bootstrap");
    boot.valence(operation).expect("valence build")
}
