//! Spectra telemetry schema registration (`spectra_metric!` / `spectra_schema!`).
//!
//! Submits metadata to [`spectra_core::SchemaRegistry`] via inventory when the `pion`
//! crate is linked. Does not depend on the `spectra` runtime crate.

mod pion_heartbeat_log_schema {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/schemas/spectra/pion_heartbeat_log_spectra_schema.rs"
    ));
}

mod pion_heartbeat_publishes_metric {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/schemas/spectra/pion_heartbeat_publishes_spectra_metric.rs"
    ));
}

mod pion_heartbeats_ingested_metric {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/schemas/spectra/pion_heartbeats_ingested_spectra_metric.rs"
    ));
}

mod pion_handoff_directives_metric {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/schemas/spectra/pion_handoff_directives_spectra_metric.rs"
    ));
}

mod pion_handoff_log_schema {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/schemas/spectra/pion_handoff_log_spectra_schema.rs"
    ));
}

mod pion_migration_log_schema {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/schemas/spectra/pion_migration_log_spectra_schema.rs"
    ));
}

mod pion_migration_runs_metric {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/schemas/spectra/pion_migration_runs_spectra_metric.rs"
    ));
}

#[allow(clippy::too_many_arguments)] // spectra schema macro emits row builders for every field
mod pion_node_action_log_schema {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/schemas/spectra/pion_node_action_log_spectra_schema.rs"
    ));
}

mod pion_node_actions_metric {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/schemas/spectra/pion_node_actions_spectra_metric.rs"
    ));
}

mod pion_server_boot_log_schema {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/schemas/spectra/pion_server_boot_log_spectra_schema.rs"
    ));
}

mod pion_server_boot_steps_metric {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/schemas/spectra/pion_server_boot_steps_spectra_metric.rs"
    ));
}
