//! Optional NDJSON sink install for headless `pion-server` and integration tests.

use std::path::Path;
use std::sync::Arc;

use spectra_core::{NdjsonFileSink, SpectraSink};

/// Install an NDJSON-only [`spectra_core`] sink when `DATA_DIR` or site-root layout is present.
///
/// # Errors
///
/// Propagates I/O errors from directory creation or sink initialization.
pub fn install_ndjson_sink_from_env() -> anyhow::Result<()> {
    let data_dir = std::env::var("DATA_DIR")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            std::env::var("LEPTOS_SITE_ROOT")
                .ok()
                .filter(|s| !s.trim().is_empty())
        })
        .unwrap_or_else(|| ".".to_string());
    install_ndjson_sink(Path::new(&data_dir))
}

/// Install NDJSON sink under `{data_dir}/spectra/`.
///
/// # Errors
///
/// Propagates I/O errors from directory creation or sink initialization.
pub fn install_ndjson_sink(data_dir: &Path) -> anyhow::Result<()> {
    let spectra_dir = data_dir.join("spectra");
    std::fs::create_dir_all(&spectra_dir)?;
    let sink = NdjsonFileSink::new(
        spectra_dir.join("metrics.ndjson"),
        spectra_dir.join("events.ndjson"),
    )?;
    spectra_core::set_sink(Arc::new(sink) as Arc<dyn SpectraSink>);
    tracing::info!(
        "[pion:logging] spectra-core NDJSON sink at {}",
        spectra_dir.display()
    );
    Ok(())
}
