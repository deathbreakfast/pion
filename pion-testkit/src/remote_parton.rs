//! HTTP client for a **remote** Parton agent, as an alternative to the in-process
//! [`crate::runner::ScenarioRunner`] `docker-cli` executor.
//!
//! Today [`crate::matrix::Executor::DockerCli`] executes container actions in-process, in the
//! same host/process as the CP simulation. That is fast and simple, but it cannot exercise a
//! *real*, separately-deployed Parton agent reachable over the network from a test/bench
//! driver running elsewhere.
//!
//! Parton today is *pull-based*: an agent process polls the control plane over
//! `/actions/claim` / `/actions/result` / `/actions/extend-lease` (see `parton::command_queue`)
//! and never exposes an inbound HTTP surface of its own. There is currently no production
//! "push a container action directly at an agent" endpoint anywhere in the stack.
//!
//! [`RemoteParton`] defines and consumes a minimal **proposed** contract for that missing
//! push-mode: a single `POST {base_url}/agent/execute` accepting a
//! [`ContainerActionRequest`] JSON body and returning a [`ContainerActionResponse`] JSON body,
//! optionally guarded by an `x-parton-token` shared-secret header (the same header name Parton's
//! own agent↔CP client sends today). It is configured via:
//!
//! - `PION_REMOTE_PARTON_URL` ([`REMOTE_PARTON_URL_ENV`]) — base URL of the remote agent, e.g.
//!   `http://10.0.1.9:8080`. When unset, [`RemoteParton::from_env`] returns `Ok(None)` so callers
//!   can fall back to the in-process executor.
//! - `PION_REMOTE_PARTON_TOKEN` ([`REMOTE_PARTON_TOKEN_ENV`]) — optional shared token sent as the
//!   `x-parton-token` header on every request.
//!
//! # Scope / what's *not* here
//!
//! This ships the client + a request/response contract test against an in-process Axum stand-in
//! that plays the role of the not-yet-built remote agent endpoint (see the `tests` module below).
//! It deliberately does **not**:
//!
//! - Add a `POST /agent/execute` handler to the real `parton` binary/router — that is separate,
//!   larger surface-area work (auth story, exposing it safely on agent hosts, etc.).
//! - Wire a new [`crate::matrix::Executor`] variant into [`crate::runner::ScenarioRunner`]'s
//!   per-step dispatch (`ClaimAndExecute` / `AssertContainerRunning` / `AssertContainerGone` /
//!   firehose steps all match exhaustively on `Executor` today). Doing so also needs a
//!   corresponding "assert container state" story over HTTP (e.g. a
//!   `GET /agent/inspect/{container_ref}` companion endpoint).
//!
//! Until that follow-up lands, use this client directly from a bench/e2e driver against a
//! purpose-built test double (as the tests below do), or as the starting point for implementing
//! the real endpoint on the Parton side.

use anyhow::{Context, Result};
use parton::{ContainerActionRequest, ContainerActionResponse};
use std::time::Duration;

/// Env var naming the base URL of a remote Parton agent's HTTP surface (e.g. `http://10.0.1.9:8080`).
pub const REMOTE_PARTON_URL_ENV: &str = "PION_REMOTE_PARTON_URL";
/// Env var naming an optional shared token sent as `x-parton-token` on every request.
pub const REMOTE_PARTON_TOKEN_ENV: &str = "PION_REMOTE_PARTON_TOKEN";

/// Path (relative to the configured base URL) that receives the direct-execute POST.
pub const EXECUTE_PATH: &str = "/agent/execute";

/// Configuration for [`RemoteParton`], normally sourced from
/// `PION_REMOTE_PARTON_URL` / `PION_REMOTE_PARTON_TOKEN`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAgentConfig {
    /// Base URL of the remote Parton agent (no trailing slash required).
    pub base_url: String,
    /// Optional shared token sent as `x-parton-token`.
    pub token: Option<String>,
}

impl RemoteAgentConfig {
    /// Read config from `PION_REMOTE_PARTON_URL` / `PION_REMOTE_PARTON_TOKEN`.
    ///
    /// Returns `None` (not an error) when `PION_REMOTE_PARTON_URL` is unset or blank, so callers
    /// can treat the remote-agent runner mode as opt-in and fall back to in-process execution.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var(REMOTE_PARTON_URL_ENV)
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())?;
        let token = std::env::var(REMOTE_PARTON_TOKEN_ENV)
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        Some(Self { base_url, token })
    }
}

/// HTTP client that submits a [`ContainerActionRequest`] directly to a remote Parton agent and
/// returns its [`ContainerActionResponse`].
pub struct RemoteParton {
    config: RemoteAgentConfig,
    client: reqwest::Client,
}

impl RemoteParton {
    /// Build a client for the given config.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client cannot be constructed.
    pub fn new(config: RemoteAgentConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_mins(1))
            .build()
            .context("build reqwest client for RemoteParton")?;
        Ok(Self { config, client })
    }

    /// Build a client from `PION_REMOTE_PARTON_URL` / `PION_REMOTE_PARTON_TOKEN`.
    ///
    /// Returns `Ok(None)` when the URL env var is unset (opt-in mode).
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client cannot be constructed.
    pub fn from_env() -> Result<Option<Self>> {
        match RemoteAgentConfig::from_env() {
            Some(config) => Ok(Some(Self::new(config)?)),
            None => Ok(None),
        }
    }

    /// The configured base URL (for logging/reporting).
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// POST `request` to the remote agent's [`EXECUTE_PATH`] and parse the response.
    ///
    /// # Errors
    ///
    /// Returns an error when the request cannot be sent, the remote agent responds with a
    /// non-success status, or the response body cannot be decoded as a [`ContainerActionResponse`].
    pub async fn execute_action(
        &self,
        request: &ContainerActionRequest,
    ) -> Result<ContainerActionResponse> {
        let url = format!(
            "{}{EXECUTE_PATH}",
            self.config.base_url.trim_end_matches('/')
        );
        let mut req = self.client.post(&url).json(request);
        if let Some(token) = self.config.token.as_deref() {
            req = req.header("x-parton-token", token);
        }
        let resp = req
            .send()
            .await
            .with_context(|| format!("POST {url} to remote Parton agent"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("remote Parton agent execute HTTP {status}: {body}");
        }
        resp.json::<ContainerActionResponse>()
            .await
            .context("decode remote Parton agent execute response")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use axum::http::HeaderMap;
    use axum::routing::post;
    use axum::{Json, Router};
    use parton::ContainerActionKind;
    use std::future::IntoFuture;
    use std::net::SocketAddr;
    use std::sync::{Arc, Mutex};
    use tokio::sync::oneshot;

    #[derive(Clone, Default)]
    struct RecordedRequest {
        token: Arc<Mutex<Option<String>>>,
        container_ref: Arc<Mutex<Option<String>>>,
    }

    async fn execute_handler(
        State(state): State<RecordedRequest>,
        headers: HeaderMap,
        Json(body): Json<ContainerActionRequest>,
    ) -> Json<ContainerActionResponse> {
        *state.token.lock().expect("lock") = headers
            .get("x-parton-token")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        *state.container_ref.lock().expect("lock") = Some(body.container_ref.clone());
        Json(ContainerActionResponse {
            action: body.action,
            container_ref: body.container_ref,
            success: true,
            message: "ok".to_string(),
            payload: serde_json::json!({"executed_by": "remote-stub"}),
        })
    }

    async fn failing_handler() -> (axum::http::StatusCode, &'static str) {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "boom")
    }

    async fn spawn_test_server(
        router: Router,
    ) -> Result<(String, oneshot::Sender<()>, tokio::task::JoinHandle<()>)> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr: SocketAddr = listener.local_addr()?;
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let server = axum::serve(listener, router).with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        });
        let handle = tokio::spawn(async move {
            let _ = server.into_future().await;
        });
        Ok((format!("http://{addr}"), shutdown_tx, handle))
    }

    fn sample_request() -> ContainerActionRequest {
        ContainerActionRequest {
            node_id: "node-1".to_string(),
            container_ref: "svc-a".to_string(),
            action: ContainerActionKind::Start,
            tail_lines: None,
            image_ref: None,
            env_vars: vec![],
            secret_env_vars: vec![],
            port_mappings: vec![],
            extra_hosts: vec![],
            entrypoint: None,
            command: vec![],
            restart_policy: None,
            volume_mounts: vec![],
            resource_limits: None,
            health_check: None,
            labels: std::collections::HashMap::new(),
            network: None,
            diagnostic: None,
            wireguard_peer: None,
            grow_fs: None,
            templated_exec: None,
            expected_container_id: None,
        }
    }

    #[test]
    #[serial_test::serial]
    fn config_from_env_none_when_url_unset() {
        std::env::remove_var(REMOTE_PARTON_URL_ENV);
        assert!(RemoteAgentConfig::from_env().is_none());
    }

    #[test]
    #[serial_test::serial]
    fn config_from_env_reads_url_and_token() {
        std::env::set_var(REMOTE_PARTON_URL_ENV, "http://127.0.0.1:9999");
        std::env::set_var(REMOTE_PARTON_TOKEN_ENV, "shared-secret");
        let cfg = RemoteAgentConfig::from_env().expect("config present");
        assert_eq!(cfg.base_url, "http://127.0.0.1:9999");
        assert_eq!(cfg.token.as_deref(), Some("shared-secret"));
        std::env::remove_var(REMOTE_PARTON_URL_ENV);
        std::env::remove_var(REMOTE_PARTON_TOKEN_ENV);
    }

    #[tokio::test]
    async fn execute_action_sends_token_and_decodes_response() -> Result<()> {
        let state = RecordedRequest::default();
        let app = Router::new()
            .route(EXECUTE_PATH, post(execute_handler))
            .with_state(state.clone());
        let (base, shutdown_tx, handle) = spawn_test_server(app).await?;

        let client = RemoteParton::new(RemoteAgentConfig {
            base_url: base,
            token: Some("shared-secret".to_string()),
        })?;
        let response = client.execute_action(&sample_request()).await?;
        assert!(response.success);
        assert_eq!(response.container_ref, "svc-a");
        assert_eq!(response.payload["executed_by"], "remote-stub");
        assert_eq!(
            state.token.lock().expect("lock").clone(),
            Some("shared-secret".to_string())
        );
        assert_eq!(
            state.container_ref.lock().expect("lock").clone(),
            Some("svc-a".to_string())
        );

        let _ = shutdown_tx.send(());
        let _ = handle.await;
        Ok(())
    }

    #[tokio::test]
    async fn execute_action_omits_token_header_when_not_configured() -> Result<()> {
        let state = RecordedRequest::default();
        let app = Router::new()
            .route(EXECUTE_PATH, post(execute_handler))
            .with_state(state.clone());
        let (base, shutdown_tx, handle) = spawn_test_server(app).await?;

        let client = RemoteParton::new(RemoteAgentConfig {
            base_url: base,
            token: None,
        })?;
        client.execute_action(&sample_request()).await?;
        assert_eq!(state.token.lock().expect("lock").clone(), None);

        let _ = shutdown_tx.send(());
        let _ = handle.await;
        Ok(())
    }

    #[tokio::test]
    async fn execute_action_fails_closed_on_non_success_status() -> Result<()> {
        let app = Router::new().route(EXECUTE_PATH, post(failing_handler));
        let (base, shutdown_tx, handle) = spawn_test_server(app).await?;

        let client = RemoteParton::new(RemoteAgentConfig {
            base_url: base,
            token: None,
        })?;
        let err = client
            .execute_action(&sample_request())
            .await
            .expect_err("non-success status must surface as an error");
        assert!(err.to_string().contains("500"));

        let _ = shutdown_tx.send(());
        let _ = handle.await;
        Ok(())
    }
}
