//! Photon topics for first-run setup progress driven by `PionNodeActionCommand`
//! `correlation_key` prefixes (`fleet.bootstrap.*.updated`).
//!
//! Pion keeps opaque `correlation_key` handling here and adds no product-branded topics.

/// Published when a DB infrastructure stack action (PD/TiKV) is enqueued or transitions.
#[cfg(feature = "runtime")]
#[photon::topic(name = "fleet.bootstrap.db_infra.updated")]
pub struct PionBootstrapDbInfraUpdated {
    /// Bootstrap session record id.
    pub bootstrap_session_id: String,
    /// Target cell id.
    pub cell_id: String,
    /// Phase/status payload (`pending:…`, `succeeded:…`, …).
    pub status: String,
}

/// Application and service deploy steps (e.g. `HAProxy` gateway per cell).
#[cfg(feature = "runtime")]
#[photon::topic(name = "fleet.bootstrap.service_deploy.updated")]
pub struct PionBootstrapServiceDeployUpdated {
    /// Bootstrap session record id.
    pub bootstrap_session_id: String,
    /// Target cell id.
    pub cell_id: String,
    /// Phase/status payload.
    pub status: String,
}

/// Host capacity / hardware probe or DNS verification from an agent.
#[cfg(feature = "runtime")]
#[photon::topic(name = "fleet.bootstrap.capacity.updated")]
pub struct PionBootstrapCapacityUpdated {
    /// Bootstrap session record id.
    pub bootstrap_session_id: String,
    /// Target cell id.
    pub cell_id: String,
    /// Phase/status payload.
    pub status: String,
}

/// Inter-cell reachability or related matrix probes.
#[cfg(feature = "runtime")]
#[photon::topic(name = "fleet.bootstrap.intercell_probe.updated")]
pub struct PionBootstrapIntercellProbeUpdated {
    /// Bootstrap session record id.
    pub bootstrap_session_id: String,
    /// Target cell id.
    pub cell_id: String,
    /// Phase/status payload.
    pub status: String,
}

/// Default bootstrap session id.
#[cfg(feature = "runtime")]
const DEFAULT_BOOTSTRAP_SESSION: &str = "default";

/// Publishes the right `fleet.bootstrap.*.updated` topic for known `correlation_key` prefixes
/// (best-effort; errors are discarded). `phase` is `pending`, `running`, `succeeded`, `failed`, etc.
#[cfg(feature = "runtime")]
pub async fn maybe_publish_bootstrap_tracked_photon(correlation_key: &str, phase: impl AsRef<str>) {
    let ck = correlation_key.trim();
    if ck.is_empty() {
        return;
    }
    let status = format!("{}:{}", phase.as_ref().trim(), ck);
    // Producers may still emit the legacy `setup_wizard_*` prefixes; match both spellings.
    let key = match ck.strip_prefix("setup_wizard_") {
        Some(rest) => format!("bootstrap_{rest}"),
        None => ck.to_string(),
    };
    let key = key.as_str();
    if let Some(rest) = key.strip_prefix("bootstrap_db_infra:") {
        let cell_id = rest.to_string();
        let _ = PionBootstrapDbInfraUpdated {
            bootstrap_session_id: DEFAULT_BOOTSTRAP_SESSION.to_string(),
            cell_id,
            status,
        }
        .publish()
        .await;
        return;
    }
    if let Some(rest) = key.strip_prefix("bootstrap_service_deploy:") {
        let cell_id = rest.to_string();
        let _ = PionBootstrapServiceDeployUpdated {
            bootstrap_session_id: DEFAULT_BOOTSTRAP_SESSION.to_string(),
            cell_id,
            status,
        }
        .publish()
        .await;
        return;
    }
    if let Some(rest) = ck.strip_prefix("host_hw_probe:") {
        // `host_hw_probe:{cell_id}:{node_id}` — use cell for UI aggregation.
        let cell_id = rest.split(':').next().unwrap_or("default").to_string();
        let _ = PionBootstrapCapacityUpdated {
            bootstrap_session_id: DEFAULT_BOOTSTRAP_SESSION.to_string(),
            cell_id,
            status,
        }
        .publish()
        .await;
        return;
    }
    if let Some(_rest) = key.strip_prefix("bootstrap_intercell_probe:") {
        // Key encodes a pair: `a:b` after the prefix; ship whole key as cell_id for matrix lookup.
        let _ = PionBootstrapIntercellProbeUpdated {
            bootstrap_session_id: DEFAULT_BOOTSTRAP_SESSION.to_string(),
            cell_id: ck.to_string(),
            status,
        }
        .publish()
        .await;
        return;
    }
    if key.starts_with("bootstrap_dns_probe:") {
        let _ = PionBootstrapCapacityUpdated {
            bootstrap_session_id: DEFAULT_BOOTSTRAP_SESSION.to_string(),
            cell_id: "global_dns".to_string(),
            status,
        }
        .publish()
        .await;
    }
}
