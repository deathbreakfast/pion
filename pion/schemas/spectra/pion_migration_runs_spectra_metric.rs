use spectra_macros::spectra_metric;

spectra_metric! {
    PionMigrationRuns {
        store: "pion",
        name: "pion_migration_runs",
        version: "0.1.0",
        description: "One-off migration script table operations. Labels: script, outcome.",
    }
}
