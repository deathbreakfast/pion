//! Typed errors for the public node-action queue APIs (enqueue / claim / report / lease).

use thiserror::Error;

/// Matchable failures from enqueue, claim, report, and lease-extension APIs.
///
/// Callers (HTTP handlers, orchestrators) can `match` on these variants instead of
/// string-parsing an opaque [`anyhow::Error`]. Storage / Valence failures are wrapped in
/// [`NodeActionError::Internal`] so the public surface stays closed. Because this type
/// implements [`std::error::Error`], it converts into `anyhow::Error` via `?`.
#[derive(Debug, Error)]
pub enum NodeActionError {
    /// Target node row is missing from inventory.
    #[error("node '{node_id}' not found")]
    NodeNotFound {
        /// Missing node id.
        node_id: String,
    },
    /// Node exists but cannot accept actions in its current status (offline / not ready).
    #[error(
        "node '{node_id}' cannot accept actions while status is '{status}' (agent is offline or not ready)"
    )]
    NodeIneligible {
        /// Node id that was refused.
        node_id: String,
        /// Current status wire value.
        status: String,
    },
    /// Action kind is disabled for this node in the capability map.
    #[error("action '{action_kind}' is disabled for node '{node_id}'")]
    CapabilityDenied {
        /// Action kind that was refused.
        action_kind: String,
        /// Node id that was refused.
        node_id: String,
    },
    /// Command id is unknown (or soft-deleted).
    #[error("unknown command {command_id}")]
    CommandNotFound {
        /// Missing command id.
        command_id: String,
    },
    /// Command targets a different node than the caller claimed.
    #[error("command {command_id} targets a different node")]
    NodeMismatch {
        /// Command id whose ownership was checked.
        command_id: String,
    },
    /// Command is not in the expected `running` status for report / lease extend.
    #[error("command {command_id} is not running (got {status})")]
    NotRunning {
        /// Command id.
        command_id: String,
        /// Actual status debug string.
        status: String,
    },
    /// Command is already terminal (`succeeded` / `failed`).
    #[error("command {command_id} already terminal ({status})")]
    AlreadyTerminal {
        /// Command id.
        command_id: String,
        /// Terminal status debug string.
        status: String,
    },
    /// Report attempt number does not match the claimed attempt.
    #[error("command {command_id} attempt mismatch (expected {expected}, got {got})")]
    AttemptMismatch {
        /// Command id.
        command_id: String,
        /// Attempt currently recorded on the command.
        expected: i64,
        /// Attempt supplied in the report body.
        got: i64,
    },
    /// Enqueue called without a non-empty correlation key / sequence.
    #[error("{0}")]
    InvalidEnqueueKey(&'static str),
    /// Unexpected storage / Valence / serialization failure.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl NodeActionError {
    /// Suggested HTTP status for agent-facing handlers that map this error.
    #[must_use]
    pub fn http_status_hint(&self) -> u16 {
        match self {
            Self::NodeNotFound { .. } | Self::CommandNotFound { .. } => 404,
            Self::CapabilityDenied { .. } | Self::NodeMismatch { .. } => 403,
            Self::NodeIneligible { .. }
            | Self::NotRunning { .. }
            | Self::AlreadyTerminal { .. }
            | Self::AttemptMismatch { .. }
            | Self::InvalidEnqueueKey(_) => 400,
            Self::Internal(_) => 500,
        }
    }
}

impl From<valence::Error> for NodeActionError {
    fn from(value: valence::Error) -> Self {
        Self::Internal(anyhow::Error::from(value))
    }
}
