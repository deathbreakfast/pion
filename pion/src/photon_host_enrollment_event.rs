//! Photon topic for host enrollment changes (for a live enrollment UI).
//!
//! Topic: `fleet.bootstrap.host_enrollment.updated`. Valence model:
//! [`crate::generated::PionAgentHostEnrollment`].

/// Payload published when a host enrollment row is created, claimed, or revoked.
#[cfg(feature = "runtime")]
#[photon::topic(name = "fleet.bootstrap.host_enrollment.updated")]
pub struct PionBootstrapHostEnrollmentUpdated {
    /// Bootstrap session record id (often [`crate::DEFAULT_BOOTSTRAP_SESSION_RECORD_ID`]).
    pub bootstrap_session_id: String,
    /// Enrollment row id.
    pub enrollment_id: String,
    /// Wire status string (`pending`, `claimed`, `revoked`, …).
    pub status: String,
}

/// Publishes the `fleet.bootstrap.host_enrollment.updated` Photon payload (best-effort; errors
/// are discarded).
#[cfg(feature = "runtime")]
pub async fn publish_bootstrap_host_enrollment_updated(
    bootstrap_session_id: impl Into<String>,
    enrollment_id: impl Into<String>,
    status: impl Into<String>,
) {
    let bootstrap_session_id = bootstrap_session_id.into();
    let enrollment_id = enrollment_id.into();
    let status = status.into();
    let _ = PionBootstrapHostEnrollmentUpdated {
        bootstrap_session_id,
        enrollment_id,
        status,
    }
    .publish()
    .await;
}
