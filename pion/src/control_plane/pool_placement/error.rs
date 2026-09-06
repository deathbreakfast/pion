//! Typed failures from virtual pool node resolution.

use thiserror::Error;

use super::hardware::UnmetRequirement;

/// Matchable reasons a pool could not produce a placement candidate.
///
/// Callers that only need a message can convert into [`anyhow::Error`] with `?`. Callers that
/// drive operator UI or retry decisions should match: [`PoolResolutionError::NoMatchingHardware`]
/// means the pool is correctly configured but currently too small, whereas
/// [`PoolResolutionError::InvalidSelector`] means an operator has to fix the pool row.
///
/// The messages for the pre-existing failure modes (missing pool, no enabled cells, no reachable
/// nodes) are unchanged from before selectors existed.
#[derive(Debug, Error)]
pub enum PoolResolutionError {
    /// No virtual pool row with this id.
    #[error("pool '{pool_id}' not found")]
    PoolNotFound {
        /// Pool id that was requested.
        pool_id: String,
    },
    /// Pool exists but maps to no enabled cell, so it has no node population.
    #[error("pool '{pool_id}' has no enabled cell mappings")]
    NoEnabledCells {
        /// Pool id that was requested.
        pool_id: String,
    },
    /// `selector_json` carries a `hardware` block that could not be parsed.
    #[error("pool '{pool_id}' has an invalid hardware selector: {source}")]
    InvalidSelector {
        /// Pool id whose selector is malformed.
        pool_id: String,
        /// Underlying deserialization failure.
        #[source]
        source: serde_json::Error,
    },
    /// `placement_policy_json` names an unknown strategy or is otherwise malformed.
    #[error("pool '{pool_id}' has an invalid placement policy: {source}")]
    InvalidPlacementPolicy {
        /// Pool id whose placement policy is malformed.
        pool_id: String,
        /// Underlying deserialization failure.
        #[source]
        source: serde_json::Error,
    },
    /// Cells are mapped, but no node in them is reachable for deploy work.
    #[error("pool '{pool_id}' has no online or draining nodes in enabled cells")]
    NoEligibleNodes {
        /// Pool id that was requested.
        pool_id: String,
    },
    /// Reachable nodes exist, but none clear the pool selector and workload floor.
    #[error(
        "pool '{pool_id}' has {considered} reachable node(s) in enabled cells but none satisfy the hardware requirements ({})",
        summarize_unmet(unmet)
    )]
    NoMatchingHardware {
        /// Pool id that was requested.
        pool_id: String,
        /// How many reachable nodes were evaluated.
        considered: usize,
        /// One failed requirement per evaluated node, in evaluation order.
        unmet: Vec<UnmetRequirement>,
    },
    /// A matching node row has no stable primary key, so it cannot be targeted.
    #[error("selected node in pool '{pool_id}' does not have a stable id")]
    UnstableNodeId {
        /// Pool id whose candidate row is unusable.
        pool_id: String,
    },
    /// Storage / Valence failure while reading pool, mapping, or node rows.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

/// Renders the distinct unmet reasons so an operator sees *why* a pool came up empty without the
/// message growing with the node count.
fn summarize_unmet(unmet: &[UnmetRequirement]) -> String {
    if unmet.is_empty() {
        return "no reason recorded".to_string();
    }
    let mut distinct: Vec<String> = Vec::new();
    for reason in unmet {
        let rendered = reason.to_string();
        if !distinct.contains(&rendered) {
            distinct.push(rendered);
        }
    }
    distinct.join("; ")
}

impl From<valence::Error> for PoolResolutionError {
    fn from(value: valence::Error) -> Self {
        Self::Internal(anyhow::Error::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control_plane::pool_placement::hardware::CpuArchitecture;

    #[test]
    fn legacy_failure_messages_are_unchanged() {
        assert_eq!(
            PoolResolutionError::PoolNotFound {
                pool_id: "edge-west".to_string()
            }
            .to_string(),
            "pool 'edge-west' not found"
        );
        assert_eq!(
            PoolResolutionError::NoEnabledCells {
                pool_id: "edge-west".to_string()
            }
            .to_string(),
            "pool 'edge-west' has no enabled cell mappings"
        );
        assert_eq!(
            PoolResolutionError::NoEligibleNodes {
                pool_id: "edge-west".to_string()
            }
            .to_string(),
            "pool 'edge-west' has no online or draining nodes in enabled cells"
        );
    }

    #[test]
    fn no_matching_hardware_deduplicates_reasons() {
        let err = PoolResolutionError::NoMatchingHardware {
            pool_id: "edge-west".to_string(),
            considered: 3,
            unmet: vec![
                UnmetRequirement::CpuLogical {
                    required: 8,
                    reported: Some(2),
                },
                UnmetRequirement::CpuLogical {
                    required: 8,
                    reported: Some(2),
                },
                UnmetRequirement::Architecture {
                    accepted: vec![CpuArchitecture::Aarch64],
                    reported: Some(CpuArchitecture::X86_64),
                },
            ],
        };
        assert_eq!(
            err.to_string(),
            "pool 'edge-west' has 3 reachable node(s) in enabled cells but none satisfy the hardware requirements (cpu_logical 2 < 8; arch x86_64 != aarch64)"
        );
    }
}
