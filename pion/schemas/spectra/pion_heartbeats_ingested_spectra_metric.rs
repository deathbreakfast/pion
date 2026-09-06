use spectra_macros::spectra_metric;

spectra_metric! {
    PionHeartbeatsIngested {
        store: "pion",
        name: "pion_heartbeats_ingested",
        version: "0.1.0",
        description: "Parton heartbeat ingest outcomes. Labels: outcome (ok, error).",
    }
}
