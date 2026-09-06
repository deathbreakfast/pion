//! The four Parton agent HTTP handlers (heartbeat ingest, claim, extend-lease, report) mounted
//! by [`super::parton_router`].

use axum::extract::connect_info::ConnectInfo;
use axum::extract::State as AxumState;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use std::net::SocketAddr;
use valence::{Actor, Valence};

use super::auth::{authorize_agent_request, authorize_node_binding, check_agent_rate_limit};
use super::HasValenceRouter;

fn valence_for_operation<S: HasValenceRouter>(
    app_state: &S,
    operation: &str,
) -> Result<Valence, String> {
    Valence::builder()
        .database_router(app_state.valence_router())
        .default_backend_key(app_state.default_backend_key().to_string())
        .with_actor(Actor::System {
            operation: operation.to_string(),
        })
        .build()
        .map_err(|error| format!("Failed to build Valence: {error}"))
}

#[tracing::instrument(
    skip(addr, app_state, headers, report),
    fields(node_id = %report.node_id, cell_id = %report.cell_id)
)]
pub(super) async fn ingest_agent_heartbeat_handler<S: HasValenceRouter>(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    AxumState(app_state): AxumState<S>,
    headers: HeaderMap,
    Json(report): Json<parton::NodeHeartbeatReport>,
) -> impl IntoResponse {
    let peer_ip = addr.ip().to_string();
    if let Err((status, message)) = check_agent_rate_limit(&peer_ip, &report.node_id) {
        return (status, message).into_response();
    }
    if let Err((status, message)) = authorize_agent_request(&headers, &report.node_id) {
        return (status, message).into_response();
    }
    if let Err((status, message)) = authorize_node_binding(&headers, &report.node_id) {
        return (status, message).into_response();
    }

    let valence = match valence_for_operation(&app_state, "parton_heartbeat_ingest") {
        Ok(valence) => valence,
        Err(message) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, message).into_response();
        }
    };

    match crate::ingest_agent_heartbeat(&report, Some(peer_ip.as_str()), &valence).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to ingest heartbeat: {error}"),
        )
            .into_response(),
    }
}

#[derive(serde::Deserialize)]
pub(super) struct AgentClaimActionBody {
    node_id: String,
    lease_duration_secs: Option<u64>,
}

#[tracing::instrument(skip(addr, app_state, headers, body), fields(node_id = %body.node_id))]
pub(super) async fn parton_claim_action_handler<S: HasValenceRouter>(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    AxumState(app_state): AxumState<S>,
    headers: HeaderMap,
    Json(body): Json<AgentClaimActionBody>,
) -> impl IntoResponse {
    let peer_ip = addr.ip().to_string();
    if let Err((status, message)) = check_agent_rate_limit(&peer_ip, &body.node_id) {
        return (status, message).into_response();
    }
    if let Err((status, message)) = authorize_agent_request(&headers, &body.node_id) {
        return (status, message).into_response();
    }
    if let Err((status, message)) = authorize_node_binding(&headers, &body.node_id) {
        return (status, message).into_response();
    }

    let valence = match valence_for_operation(&app_state, "parton_action_claim") {
        Ok(valence) => valence,
        Err(message) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, message).into_response();
        }
    };

    let lease = body
        .lease_duration_secs
        .unwrap_or_else(crate::default_lease_duration_secs)
        .max(1);
    match crate::claim_pending_node_action(&body.node_id, lease, &valence).await {
        Ok(None) => StatusCode::NO_CONTENT.into_response(),
        Ok(Some(cmd)) => Json(cmd).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to claim action: {error}"),
        )
            .into_response(),
    }
}

#[derive(serde::Deserialize)]
pub(super) struct AgentExtendLeaseBody {
    command_id: String,
    node_id: String,
    additional_secs: Option<u64>,
}

#[tracing::instrument(
    skip(addr, app_state, headers, body),
    fields(command_id = %body.command_id, node_id = %body.node_id)
)]
pub(super) async fn parton_extend_action_lease_handler<S: HasValenceRouter>(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    AxumState(app_state): AxumState<S>,
    headers: HeaderMap,
    Json(body): Json<AgentExtendLeaseBody>,
) -> impl IntoResponse {
    let peer_ip = addr.ip().to_string();
    if let Err((status, message)) = check_agent_rate_limit(&peer_ip, &body.node_id) {
        return (status, message).into_response();
    }
    if let Err((status, message)) = authorize_agent_request(&headers, &body.node_id) {
        return (status, message).into_response();
    }
    if let Err((status, message)) = authorize_node_binding(&headers, &body.node_id) {
        return (status, message).into_response();
    }

    let valence = match valence_for_operation(&app_state, "parton_action_extend_lease") {
        Ok(valence) => valence,
        Err(message) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, message).into_response();
        }
    };

    let add = body.additional_secs.unwrap_or(300).max(1);
    match crate::extend_node_action_lease(&body.command_id, &body.node_id, add, &valence).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => {
            let status =
                StatusCode::from_u16(error.http_status_hint()).unwrap_or(StatusCode::BAD_REQUEST);
            (status, format!("Failed to extend action lease: {error}")).into_response()
        }
    }
}

#[tracing::instrument(
    skip(addr, app_state, headers, body),
    fields(command_id = %body.command_id, node_id = %body.node_id)
)]
pub(super) async fn parton_report_action_handler<S: HasValenceRouter>(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    AxumState(app_state): AxumState<S>,
    headers: HeaderMap,
    Json(body): Json<crate::ReportNodeActionResult>,
) -> impl IntoResponse {
    let peer_ip = addr.ip().to_string();
    if let Err((status, message)) = check_agent_rate_limit(&peer_ip, &body.node_id) {
        return (status, message).into_response();
    }
    if let Err((status, message)) = authorize_agent_request(&headers, &body.node_id) {
        return (status, message).into_response();
    }
    if let Err((status, message)) = authorize_node_binding(&headers, &body.node_id) {
        return (status, message).into_response();
    }

    let valence = match valence_for_operation(&app_state, "parton_action_result") {
        Ok(valence) => valence,
        Err(message) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, message).into_response();
        }
    };

    match crate::report_node_action_result(body, &valence).await {
        Ok(()) => StatusCode::ACCEPTED.into_response(),
        Err(error) => {
            let status =
                StatusCode::from_u16(error.http_status_hint()).unwrap_or(StatusCode::BAD_REQUEST);
            (status, format!("Failed to record action result: {error}")).into_response()
        }
    }
}
