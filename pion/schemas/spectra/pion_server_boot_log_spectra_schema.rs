use spectra_macros::spectra_schema;

spectra_schema! {
    PionServerBootLog {
        store: "pion",
        table: "pion_server_boot_log",
        version: "0.1.0",
        description: "pion-server bootstrap failures and operator drill-down.",
        fields: [
            step: {
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
