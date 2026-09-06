//! Shared pull-only deployable for edge / fair-compare benches.

/// Default public nginx image (no build in the measured path).
pub const DEFAULT_BENCH_IMAGE: &str = "public.ecr.aws/nginx/nginx:alpine";

/// Env override for the bench image ref (`PION_BENCH_IMAGE`).
pub const BENCH_IMAGE_ENV: &str = "PION_BENCH_IMAGE";

/// Resolve the image ref from env or the shared default.
#[must_use]
pub fn bench_image_ref() -> String {
    std::env::var(BENCH_IMAGE_ENV)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_BENCH_IMAGE.to_string())
}

/// Resolve a registry digest for `image` via `docker image inspect`, if present locally.
///
/// # Errors
///
/// Returns an error when the Docker CLI fails unexpectedly (missing image yields `Ok(None)`).
pub fn resolve_image_digest(image: &str) -> anyhow::Result<Option<String>> {
    let output = std::process::Command::new("docker")
        .args([
            "image",
            "inspect",
            "--format",
            "{{if .RepoDigests}}{{index .RepoDigests 0}}{{end}}",
            image,
        ])
        .output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

/// Pull `image` (warm path / cold-path setup). Runs on a blocking thread pool when called via
/// [`pre_pull_image_async`].
///
/// # Errors
///
/// Returns an error when `docker pull` fails.
pub fn pre_pull_image(image: &str) -> anyhow::Result<()> {
    let output = std::process::Command::new("docker")
        .args(["pull", image])
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("docker pull {image} failed: {stderr}");
    }
    Ok(())
}

/// Async wrapper: pull on the blocking pool.
///
/// # Errors
///
/// Returns an error when the blocking task join fails or pull fails.
pub async fn pre_pull_image_async(image: String) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || pre_pull_image(&image))
        .await
        .map_err(|e| anyhow::anyhow!("pull join: {e}"))?
}

/// Host port published for single-container warm/cold deploy rows.
pub const BENCH_HOST_PORT: u16 = 18_765;

/// Format `host:container` port mapping for nginx (:80).
#[must_use]
pub fn bench_port_mapping(host_port: u16) -> String {
    format!("{host_port}:80")
}
