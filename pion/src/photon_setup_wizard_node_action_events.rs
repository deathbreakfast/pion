//! Photon topics for first-run / Host Setup progress driven by `PionNodeActionCommand`
//! `correlation_key` prefixes (`gluon.setup_wizard.*.updated`).
//!
//! **Transitional:** topic strings stay for subscriber compatibility. New fleet-bootstrap
//! progress events should use the `fleet.bootstrap.*` prefix. Pion keeps opaque `correlation_key`
//! handling here; avoid adding new product-branded topics in this crate.

/// Published when a DB infrastructure stack action (PD/TiKV/Surreal) is enqueued or transitions.
#[cfg(feature = "runtime")]
#[photon::topic(name = "gluon.setup_wizard.db_infra.updated")]
pub struct PionSetupWizardDbInfraUpdated {
    /// Setup-wizard session record id.
    pub setup_wizard_session_id: String,
    /// Target cell id.
    pub cell_id: String,
    /// Phase/status payload (`pending:…`, `succeeded:…`, …).
    pub status: String,
}

/// Gluon application / service deploy steps (e.g. `HAProxy` gateway per cell).
#[cfg(feature = "runtime")]
#[photon::topic(name = "gluon.setup_wizard.service_deploy.updated")]
pub struct PionSetupWizardServiceDeployUpdated {
    /// Setup-wizard session record id.
    pub setup_wizard_session_id: String,
    /// Target cell id.
    pub cell_id: String,
    /// Phase/status payload.
    pub status: String,
}

/// Host capacity / hardware probe or DNS verification from an agent.
#[cfg(feature = "runtime")]
#[photon::topic(name = "gluon.setup_wizard.capacity.updated")]
pub struct PionSetupWizardCapacityUpdated {
    /// Setup-wizard session record id.
    pub setup_wizard_session_id: String,
    /// Target cell id.
    pub cell_id: String,
    /// Phase/status payload.
    pub status: String,
}

/// Inter-cell reachability or related matrix probes.
#[cfg(feature = "runtime")]
#[photon::topic(name = "gluon.setup_wizard.intercell_probe.updated")]
pub struct PionSetupWizardIntercellProbeUpdated {
    /// Setup-wizard session record id.
    pub setup_wizard_session_id: String,
    /// Target cell id.
    pub cell_id: String,
    /// Phase/status payload.
    pub status: String,
}

/// Default session id (matches setup-wizard-app `DEFAULT_SETUP_SESSION_ID`).
#[cfg(feature = "runtime")]
const DEFAULT_WIZARD_SESSION: &str = "default";

/// Publishes the right `gluon.setup_wizard.*.updated` topic for known `correlation_key` prefixes
/// (best-effort; errors are discarded). `phase` is `pending`, `running`, `succeeded`, `failed`, etc.
#[cfg(feature = "runtime")]
pub async fn maybe_publish_setup_wizard_tracked_photon(
    correlation_key: &str,
    phase: impl AsRef<str>,
) {
    let ck = correlation_key.trim();
    if ck.is_empty() {
        return;
    }
    let status = format!("{}:{}", phase.as_ref().trim(), ck);
    if let Some(rest) = ck.strip_prefix("setup_wizard_db_infra:") {
        let cell_id = rest.to_string();
        let _ = PionSetupWizardDbInfraUpdated {
            setup_wizard_session_id: DEFAULT_WIZARD_SESSION.to_string(),
            cell_id,
            status,
        }
        .publish()
        .await;
        return;
    }
    if let Some(rest) = ck.strip_prefix("setup_wizard_service_deploy:") {
        let cell_id = rest.to_string();
        let _ = PionSetupWizardServiceDeployUpdated {
            setup_wizard_session_id: DEFAULT_WIZARD_SESSION.to_string(),
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
        let _ = PionSetupWizardCapacityUpdated {
            setup_wizard_session_id: DEFAULT_WIZARD_SESSION.to_string(),
            cell_id,
            status,
        }
        .publish()
        .await;
        return;
    }
    if let Some(_rest) = ck.strip_prefix("setup_wizard_intercell_probe:") {
        // Key encodes a pair: `a:b` after the prefix; ship whole key as cell_id for matrix lookup.
        let _ = PionSetupWizardIntercellProbeUpdated {
            setup_wizard_session_id: DEFAULT_WIZARD_SESSION.to_string(),
            cell_id: ck.to_string(),
            status,
        }
        .publish()
        .await;
        return;
    }
    if ck.starts_with("setup_wizard_dns_probe:") {
        let _ = PionSetupWizardCapacityUpdated {
            setup_wizard_session_id: DEFAULT_WIZARD_SESSION.to_string(),
            cell_id: "global_dns".to_string(),
            status,
        }
        .publish()
        .await;
    }
}
