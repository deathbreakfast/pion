use spectra_macros::spectra_schema;

spectra_schema! {
    PionHeartbeatLog {
        store: "pion",
        table: "pion_heartbeat_log",
        version: "0.1.0",
        description: "Parton heartbeat ingest and container transition trace.",
        fields: [
            node_id: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            cell_id: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            state: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            message: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
        ],
    }
}
