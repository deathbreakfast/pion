//! One-time host enrollment for agent heartbeat ingest (Fleet Bootstrap / local cell).

mod config;
mod error;
mod tickets;
mod token;
mod verify;

pub use config::{
    enrollment_strict_new_nodes, normalize_agent_host_hint, DEFAULT_BOOTSTRAP_SESSION_RECORD_ID,
    DEFAULT_SETUP_WIZARD_SESSION_RECORD_ID,
};
pub use error::EnrollmentError;
pub use tickets::{
    create_host_enrollment, list_enrollments_for_bootstrap_session, list_enrollments_for_session,
    revoke_host_enrollment, set_host_enrollment_pubkey,
};
pub use verify::{mark_enrollment_claimed, validate_enrollment_for_new_node};
