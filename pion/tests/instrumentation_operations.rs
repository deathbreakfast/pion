//! Pion product Spectra instrumentation (metrics/events via `RecordingSink`).
// Integration tests use `.expect()` / `.unwrap()` for brevity on fixtures.
#![allow(clippy::unwrap_used, clippy::expect_used)]
mod instrumentation_support;

use std::sync::Arc;

use pion::logging::{enqueue_noop_message, NodeActionContext};
use serial_test::serial;
use spectra_core::{set_sink, RecordingSink};

use instrumentation_support::{assert_counter, assert_event_field};

fn install_sink() -> RecordingSink {
    let sink = RecordingSink::new();
    set_sink(Arc::new(sink.clone()));
    sink
}

#[test]
#[serial]
fn enqueue_noop_emits_counter_and_event() {
    let sink = install_sink();
    let ctx = NodeActionContext::new("cmd-1", "node-a", "deploy").with_correlation("corr:1", 1);
    ctx.log_enqueue("noop", &enqueue_noop_message("pending", false));

    assert_counter(
        &sink,
        "pion_node_actions",
        &[("action", "enqueue"), ("outcome", "noop")],
    );
    assert_event_field(&sink, "pion_node_action_log", "outcome", "noop");
    assert_event_field(&sink, "pion_node_action_log", "command_id", "cmd-1");
}

#[test]
#[serial]
fn handoff_ack_skip_emits_directive_metric() {
    let sink = install_sink();
    pion::logging::HandoffDirectiveContext::from_node("node-h1").warn_ack_skipped("test ack error");

    assert_counter(
        &sink,
        "pion_handoff_directives",
        &[("phase", "ack"), ("outcome", "skipped")],
    );
    assert_event_field(&sink, "pion_handoff_log", "phase", "ack");
}

#[test]
#[serial]
fn migration_skip_emits_migration_log() {
    let sink = install_sink();
    pion::logging::migration::log_migration_skip(
        "gluon_control_plane_node",
        "pion_control_plane_node",
        3,
    );

    assert_counter(
        &sink,
        "pion_migration_runs",
        &[
            ("script", "migrate_gluon_control_plane_tables_to_pion"),
            ("outcome", "skipped"),
        ],
    );
    assert_event_field(&sink, "pion_migration_log", "outcome", "skipped");
}

#[test]
#[serial]
fn boot_warn_emits_boot_metric_and_event() {
    let sink = install_sink();
    pion::logging::warn_boot_step("nucleus_bootstrap", "bootstrap failed");

    assert_counter(
        &sink,
        "pion_server_boot_steps",
        &[("step", "nucleus_bootstrap"), ("outcome", "warn")],
    );
    assert_event_field(&sink, "pion_server_boot_log", "step", "nucleus_bootstrap");
}
