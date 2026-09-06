//! Migration script metrics/events emit helpers.

use spectra_core::{try_log_event, try_record_counter};

use super::events::migration_log_fields;

const MIGRATION_SCRIPT: &str = "migrate_gluon_control_plane_tables_to_pion";

/// Log per-table migration skip (destination already populated).
pub fn log_migration_skip(from: &str, to: &str, row_count: i64) {
    let message = format!("Skip {from} -> {to}: destination already has {row_count} row(s)");
    record_migration("skip", "skipped", from, to, 0, &message);
}

/// Log successful table copy.
pub fn log_migration_copied(from: &str, to: &str, rows: i64) {
    let message = format!("Copied {rows} row(s) {from} -> {to}");
    record_migration("copy", "ok", from, to, rows, &message);
}

/// Log per-table migration failure.
pub fn log_migration_failed(from: &str, to: &str, error: &str) {
    let message = format!("{from} -> {to}: {error}");
    record_migration("copy", "error", from, to, 0, &message);
}

fn record_migration(
    _operation: &str,
    outcome: &str,
    from: &str,
    to: &str,
    rows_copied: i64,
    message: &str,
) {
    try_record_counter(
        "pion_migration_runs",
        &[("script", MIGRATION_SCRIPT), ("outcome", outcome)],
        1,
    );
    try_log_event(
        "pion_migration_log",
        &migration_log_fields(MIGRATION_SCRIPT, from, to, rows_copied, outcome, message),
    );
}
