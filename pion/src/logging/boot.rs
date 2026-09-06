//! pion-server bootstrap metrics/events emit helpers.

use spectra_core::{try_log_event, try_record_counter};

use super::events::server_boot_log_fields;

/// Record a bootstrap step outcome (warn/error paths).
pub fn log_boot_step(step: &str, outcome: &str, message: &str) {
    try_record_counter(
        "pion_server_boot_steps",
        &[("step", step), ("outcome", outcome)],
        1,
    );
    if outcome != "ok" {
        try_log_event(
            "pion_server_boot_log",
            &server_boot_log_fields(step, message),
        );
    }
}

/// Record bootstrap failure with warn outcome.
pub fn warn_boot_step(step: &str, message: &str) {
    log_boot_step(step, "warn", message);
}
