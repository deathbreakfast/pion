use spectra_macros::spectra_metric;

spectra_metric! {
    PionHeartbeatPublishes {
        store: "pion",
        name: "pion_heartbeat_publishes",
        version: "0.1.0",
        description: "Container state Photon publish outcomes. Labels: outcome (success, skipped, error).",
    }
}
