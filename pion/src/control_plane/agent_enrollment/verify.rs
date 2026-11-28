//! Wire token verification against pending enrollment rows, plus new-node / claim entrypoints
//! consumed by heartbeat ingest.

use anyhow::{Context, Result};
use chrono::Utc;
use valence::{Model, Valence};

use crate::generated::{PionAgentHostEnrollment, PionAgentHostEnrollmentStatus};

use super::config::enrollment_verify_source_ip;
use super::error::EnrollmentError;
use super::token::{constant_time_eq, hex_sha256, parse_enrollment_wire_token};

fn host_part(expected: &str) -> String {
    expected
        .trim()
        .split_once(':')
        .map_or_else(|| expected.trim().to_string(), |(h, _)| h.to_string())
}

fn source_ip_matches_expected(peer_ip: Option<&str>, expected_agent_host: &str) -> bool {
    let Some(peer) = peer_ip.map(str::trim).filter(|s| !s.is_empty()) else {
        return false;
    };
    let exp = host_part(expected_agent_host);
    if exp.is_empty() {
        return true;
    }
    peer == exp
}

async fn load_pending_enrollment_row(
    valence: &Valence,
    enrollment_id: &str,
) -> Result<PionAgentHostEnrollment, EnrollmentError> {
    let row = PionAgentHostEnrollment::get(enrollment_id, valence, valence::use_!("When a **new agent presents an enrollment token** on its first heartbeat, we **load the enrollment ticket it names** to check it is still pending, unexpired and issued for this host. The control plane uses the answer to accept or reject the agent; the ticket is not shown to anyone."))
        .await
        .with_context(|| format!("load enrollment {enrollment_id} for verify"))
        .map_err(EnrollmentError::Internal)?
        .ok_or(EnrollmentError::UnknownId)?;
    if *row.status() != PionAgentHostEnrollmentStatus::Pending {
        return Err(EnrollmentError::NotPending);
    }
    if Utc::now() > *row.expires_at() {
        tracing::warn!(
            target: "security.enroll",
            enrollment_id = %enrollment_id,
            "enrollment verify failed: token expired"
        );
        return Err(EnrollmentError::Expired);
    }
    Ok(row)
}

fn verify_enrollment_source_ip(
    row: &PionAgentHostEnrollment,
    enrollment_id: &str,
    peer_ip: Option<&str>,
) -> Result<(), EnrollmentError> {
    if enrollment_verify_source_ip()
        && !source_ip_matches_expected(peer_ip, row.expected_agent_host())
    {
        tracing::warn!(
            target: "security.enroll",
            enrollment_id = %enrollment_id,
            peer_ip = ?peer_ip,
            expected_agent_host = %row.expected_agent_host(),
            "enrollment verify failed: source ip does not match expected host"
        );
        return Err(EnrollmentError::SourceIpMismatch);
    }
    Ok(())
}

fn verify_enrollment_token_hash(
    raw: &str,
    row: &PionAgentHostEnrollment,
    enrollment_id: &str,
) -> Result<(), EnrollmentError> {
    let expected_hash = row.token_hash();
    let got_hash = hex_sha256(raw);
    if !constant_time_eq(expected_hash, &got_hash) {
        tracing::warn!(
            target: "security.enroll",
            enrollment_id = %enrollment_id,
            "enrollment verify failed: token mismatch"
        );
        return Err(EnrollmentError::TokenMismatch);
    }
    Ok(())
}

fn effective_cell_id_for_enrollment(
    row: &PionAgentHostEnrollment,
    report_cell_id: &str,
    enrollment_id: &str,
) -> Result<String, EnrollmentError> {
    let row_cell = row.cell_id().trim();
    if !row_cell.is_empty() && row_cell != report_cell_id.trim() {
        tracing::warn!(
            target: "security.enroll",
            enrollment_id = %enrollment_id,
            "enrollment verify failed: cell_id mismatch"
        );
        return Err(EnrollmentError::CellIdMismatch);
    }
    Ok(if row_cell.is_empty() {
        report_cell_id.to_string()
    } else {
        row_cell.to_string()
    })
}

/// Validates a wire enrollment token against a **pending**, unexpired row (hash, cell, optional IP).
/// Returns `(enrollment_id, effective_cell_id)`.
async fn verify_pending_enrollment_wire(
    valence: &Valence,
    raw: &str,
    report_cell_id: &str,
    peer_ip: Option<&str>,
) -> Result<(String, String), EnrollmentError> {
    let raw = raw.trim();
    let Some((enrollment_id, _secret)) = parse_enrollment_wire_token(raw) else {
        tracing::warn!(
            target: "security.enroll",
            peer_ip = ?peer_ip,
            "enrollment verify failed: invalid token format"
        );
        return Err(EnrollmentError::InvalidFormat);
    };
    let row = load_pending_enrollment_row(valence, &enrollment_id).await?;
    verify_enrollment_source_ip(&row, &enrollment_id, peer_ip)?;
    verify_enrollment_token_hash(raw, &row, &enrollment_id)?;
    let effective = effective_cell_id_for_enrollment(&row, report_cell_id, &enrollment_id)?;
    tracing::info!(
        target: "security.enroll",
        enrollment_id = %enrollment_id,
        cell_id = %effective,
        "enrollment token verified"
    );
    Ok((enrollment_id, effective))
}

/// Longest `last_error` text stored on an enrollment row.
const LAST_ERROR_MAX_CHARS: usize = 256;

/// Whether `err` is a rejection of a known ticket worth recording as the ticket's `last_error`.
///
/// Malformed tokens, unknown ids and non-pending tickets have no row to record on (or one that
/// is already settled); [`EnrollmentError::Internal`] is a control-plane fault, not the agent's.
const fn is_recordable_rejection(err: &EnrollmentError) -> bool {
    matches!(
        err,
        EnrollmentError::Expired
            | EnrollmentError::SourceIpMismatch
            | EnrollmentError::TokenMismatch
            | EnrollmentError::CellIdMismatch
    )
}

/// Stores `err` as the pending ticket's `last_error` / `last_error_at`.
///
/// The stored text is the fixed [`EnrollmentError`] message, so it never carries the token,
/// its secret or the peer address.
async fn record_enrollment_failure(
    valence: &Valence,
    enrollment_id: &str,
    err: &EnrollmentError,
) -> Result<()> {
    let Some(row) = PionAgentHostEnrollment::get(enrollment_id, valence, valence::use_!(r"When a new agent's **first heartbeat is rejected** because its enrollment token is wrong, expired, from the wrong host or for the wrong cell, we **load the enrollment ticket it names** to check it is still pending before saving the reason."))
        .await
        .with_context(|| format!("load enrollment {enrollment_id} to record failure"))?
    else {
        return Ok(());
    };
    if *row.status() != PionAgentHostEnrollmentStatus::Pending {
        return Ok(());
    }
    let message: String = err.to_string().chars().take(LAST_ERROR_MAX_CHARS).collect();
    row.get_mutable(valence, valence::use_!(r"When a new agent's **first heartbeat is rejected**, we **save the reason on its enrollment ticket** (a fixed message such as **enrollment token mismatch**, never the token itself) with the time. Gluon's agent fleet replace shows this reason on a replacement host that never finished enrolling."))
        .set_last_error(message)?
        .set_last_error_at(Utc::now())?
        .commit()
        .await
        .with_context(|| format!("commit last_error for enrollment {enrollment_id}"))?;
    Ok(())
}

/// Validate enrollment token for a **new** node heartbeat; returns matching `cell_id` to use for ingest.
///
/// When the token names a pending ticket but is rejected (expired, wrong secret, wrong source IP
/// or wrong cell), the rejection message is also saved as the ticket's `last_error` and
/// `last_error_at`, so a caller waiting for the host to enroll can show why it did not. Saving is
/// best effort: a storage failure is logged and the original error is still returned.
///
/// # Errors
///
/// Returns [`EnrollmentError`] when the token is missing/invalid/expired, cell/IP/token checks fail,
/// or Valence persistence fails ([`EnrollmentError::Internal`]).
pub async fn validate_enrollment_for_new_node(
    valence: &Valence,
    enrollment_wire_token: Option<&str>,
    report_cell_id: &str,
    peer_ip: Option<&str>,
) -> Result<String, EnrollmentError> {
    let Some(raw) = enrollment_wire_token
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return Err(EnrollmentError::TokenRequired);
    };
    match verify_pending_enrollment_wire(valence, raw, report_cell_id, peer_ip).await {
        Ok((_id, effective)) => Ok(effective),
        Err(err) => {
            if is_recordable_rejection(&err) {
                if let Some((enrollment_id, _secret)) = parse_enrollment_wire_token(raw) {
                    if let Err(e) = record_enrollment_failure(valence, &enrollment_id, &err).await {
                        tracing::warn!(
                            target: "security.enroll",
                            enrollment_id = %enrollment_id,
                            error = %e,
                            "could not record enrollment failure on ticket"
                        );
                    }
                }
            }
            Err(err)
        }
    }
}

/// Mark enrollment claimed and pin claimed node id.
///
/// `report_cell_id` / `peer_ip` must match [`verify_pending_enrollment_wire`] (same as heartbeat ingest).
///
/// # Errors
///
/// Returns [`EnrollmentError`] on verify/storage failure. A [`EnrollmentError::NotPending`] verify
/// result is treated as success (idempotent claim).
pub async fn mark_enrollment_claimed(
    valence: &Valence,
    enrollment_wire_token: &str,
    claimed_node_id: &str,
    report_cell_id: &str,
    peer_ip: Option<&str>,
) -> Result<(), EnrollmentError> {
    let (enrollment_id, _) = match verify_pending_enrollment_wire(
        valence,
        enrollment_wire_token.trim(),
        report_cell_id,
        peer_ip,
    )
    .await
    {
        Ok(pair) => pair,
        Err(EnrollmentError::NotPending) => return Ok(()),
        Err(e) => return Err(e),
    };
    let row = PionAgentHostEnrollment::get(&enrollment_id, valence, valence::use_!(r"After a first-seen agent presents a valid **enrollment token** on heartbeat, we **load that enrollment ticket** so the control plane can mark it claimed. Control-plane ingest and enrollment consoles use this ticket state."))
        .await
        .with_context(|| format!("load enrollment {enrollment_id} to mark claimed"))?
        .ok_or(EnrollmentError::UnknownId)?;
    if *row.status() != PionAgentHostEnrollmentStatus::Pending {
        return Ok(());
    }
    let session_id_for_photon = row.bootstrap_session_id().clone();
    let now = Utc::now();
    row.get_mutable(valence, valence::use_!(r"After a first-seen agent **claims** a host enrollment ticket on heartbeat, we **mark the enrollment claimed** and pin the node id so the ticket cannot be reused. Control-plane ingest and enrollment consoles use that status."))
        .set_status(PionAgentHostEnrollmentStatus::Claimed)?
        .set_claimed_node_id(claimed_node_id.to_string())?
        .set_claimed_at(now)?
        .commit()
        .await
        .with_context(|| format!("commit claimed status for enrollment {enrollment_id}"))?;
    tracing::info!(
        target: "security.enroll",
        enrollment_id = %enrollment_id,
        claimed_node_id = %claimed_node_id,
        "enrollment claimed"
    );
    crate::publish_bootstrap_host_enrollment_updated(
        session_id_for_photon,
        enrollment_id,
        "claimed",
    )
    .await;
    Ok(())
}
