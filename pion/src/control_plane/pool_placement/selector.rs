//! Typed views over a virtual pool's stored hardware selector and placement policy.
//!
//! [`VirtualPoolSelector`] is what `PionControlPlaneVirtualPool.hardware_selector_json` holds. That
//! column is optional, and an unset column reads back as JSON null, so every pool created before it
//! existed is unconstrained: it keeps accepting every node in its mapped cells.
//!
//! [`VirtualPoolSelector::from_pool_columns`] also accepts a `hardware` key nested inside the older
//! free-form `selector_json`, which is where a pool's hardware rule lived before it had a column of
//! its own. The dedicated column wins when both carry constraints.

use serde::{Deserialize, Serialize};

use super::hardware::NodeHardwareRequirements;

/// Hardware membership rule for a virtual pool.
///
/// A pool's selector applies to *every* placement into that pool. A caller's
/// [`NodeHardwareRequirements`] applies to one workload. Both must hold for a node to be
/// eligible; the selector is the pool-wide floor and the caller's requirements are the workload
/// floor on top of it.
///
/// # JSON shape
///
/// ```json
/// { "hardware": { "arch": "aarch64", "min_cpu_logical": 4, "min_memory_bytes": 8589934592 } }
/// ```
///
/// An absent, null, or empty `hardware` key means the pool constrains nothing. The `hardware`
/// wrapper leaves room for later non-hardware membership rules without changing the column type.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualPoolSelector {
    /// Floor every node in the pool must clear.
    #[serde(default)]
    pub hardware: NodeHardwareRequirements,
}

impl VirtualPoolSelector {
    /// Parses a pool's `hardware_selector_json` column.
    ///
    /// JSON null (what an unset optional column reads back as) and `{}` both parse to the
    /// unconstrained selector, which is what makes the column safe to add to existing pool rows.
    ///
    /// # Errors
    ///
    /// Returns the deserialization error when `hardware` is present but malformed, for example a
    /// `min_cpu_logical` written as a string. A malformed constraint is refused rather than
    /// dropped, so a typo cannot quietly widen a pool.
    pub fn from_hardware_selector_json(
        value: &serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        if value.is_null() {
            return Ok(Self::default());
        }
        serde_json::from_value(value.clone())
    }

    /// Reads the effective selector from both pool columns.
    ///
    /// `hardware_selector_json` is the column to write. `selector_json` is consulted only when the
    /// dedicated column is unconstrained, which keeps pools that stored their hardware rule under
    /// the older `selector_json.hardware` key working. Keys in `selector_json` other than
    /// `hardware` are ignored, so a pool carrying UI state such as `{"strategy":"manual-ui"}`
    /// stays unconstrained.
    ///
    /// # Errors
    ///
    /// Returns the deserialization error from whichever column is malformed.
    pub fn from_pool_columns(
        hardware_selector_json: &serde_json::Value,
        selector_json: &serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        let dedicated = Self::from_hardware_selector_json(hardware_selector_json)?;
        if !dedicated.is_unconstrained() {
            return Ok(dedicated);
        }
        Self::from_selector_json(selector_json)
    }

    /// Parses a hardware selector nested in the older free-form `selector_json` column.
    ///
    /// # Errors
    ///
    /// Returns the deserialization error when the embedded `hardware` block is malformed.
    pub fn from_selector_json(value: &serde_json::Value) -> Result<Self, serde_json::Error> {
        if value.is_null() {
            return Ok(Self::default());
        }
        serde_json::from_value(value.clone())
    }

    /// True when the pool accepts every node in its mapped cells.
    #[must_use]
    pub fn is_unconstrained(&self) -> bool {
        self.hardware.is_unconstrained()
    }
}

/// Order in which eligible pool nodes are preferred.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PoolPlacementStrategy {
    /// Fewest reported running containers first. The historical and default behavior.
    #[default]
    LeastLoaded,
    /// Most logical CPUs first, then fewest running containers.
    MostCpu,
    /// Most total memory first, then fewest running containers.
    MostMemory,
}

impl PoolPlacementStrategy {
    /// Stable wire value used in `placement_policy_json`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LeastLoaded => "least-loaded",
            Self::MostCpu => "most-cpu",
            Self::MostMemory => "most-memory",
        }
    }
}

/// How a virtual pool ranks the nodes that pass its selector.
///
/// # JSON shape
///
/// ```json
/// { "strategy": "most-memory" }
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualPoolPlacementPolicy {
    /// Preference order applied to eligible nodes.
    #[serde(default)]
    pub strategy: PoolPlacementStrategy,
}

impl VirtualPoolPlacementPolicy {
    /// Parses a pool's `placement_policy_json` column.
    ///
    /// A JSON null or `{}` parses to [`PoolPlacementStrategy::LeastLoaded`], preserving the
    /// ordering pool deploys used before this type existed.
    ///
    /// # Errors
    ///
    /// Returns the deserialization error for an unrecognized `strategy`. Ranking silently is
    /// worse than refusing, because an operator who asked for memory-first placement would
    /// otherwise get load-first placement with no signal.
    pub fn from_placement_policy_json(
        value: &serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        if value.is_null() {
            return Ok(Self::default());
        }
        serde_json::from_value(value.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control_plane::pool_placement::hardware::CpuArchitecture;

    #[test]
    fn legacy_selector_json_parses_as_unconstrained() {
        let selector = VirtualPoolSelector::from_selector_json(
            &serde_json::json!({ "strategy": "manual-ui" }),
        )
        .expect("legacy selector parses");
        assert!(selector.is_unconstrained());
        assert_eq!(selector, VirtualPoolSelector::default());
    }

    #[test]
    fn null_and_empty_selector_json_parse_as_unconstrained() {
        assert!(
            VirtualPoolSelector::from_selector_json(&serde_json::Value::Null)
                .expect("null selector parses")
                .is_unconstrained()
        );
        assert!(
            VirtualPoolSelector::from_selector_json(&serde_json::json!({}))
                .expect("empty selector parses")
                .is_unconstrained()
        );
        assert!(
            VirtualPoolSelector::from_selector_json(&serde_json::json!({ "hardware": {} }))
                .expect("empty hardware parses")
                .is_unconstrained()
        );
    }

    #[test]
    fn hardware_selector_parses_typed_constraints() {
        let selector = VirtualPoolSelector::from_selector_json(&serde_json::json!({
            "strategy": "manual-ui",
            "hardware": { "arch": "arm64", "min_cpu_logical": 8, "min_memory_bytes": 34_359_738_368_u64 },
        }))
        .expect("hardware selector parses");
        assert!(!selector.is_unconstrained());
        assert_eq!(
            selector.hardware.architectures,
            vec![CpuArchitecture::Aarch64]
        );
        assert_eq!(selector.hardware.min_cpu_logical, Some(8));
        assert_eq!(selector.hardware.min_memory_bytes, Some(34_359_738_368));
    }

    #[test]
    fn malformed_hardware_selector_is_refused() {
        let err = VirtualPoolSelector::from_selector_json(
            &serde_json::json!({ "hardware": { "min_memory_bytes": "lots" } }),
        )
        .expect_err("malformed selector must not parse as unconstrained");
        assert!(
            err.to_string().contains("invalid type"),
            "error should explain the type mismatch, got: {err}"
        );
    }

    #[test]
    fn unset_hardware_selector_column_is_unconstrained() {
        for value in [serde_json::Value::Null, serde_json::json!({})] {
            assert!(VirtualPoolSelector::from_hardware_selector_json(&value)
                .expect("empty column parses")
                .is_unconstrained());
        }
    }

    #[test]
    fn hardware_selector_column_is_read_ahead_of_selector_json() {
        let selector = VirtualPoolSelector::from_pool_columns(
            &serde_json::json!({ "hardware": { "arch": "x86_64" } }),
            &serde_json::json!({ "hardware": { "arch": "arm64" } }),
        )
        .expect("both columns parse");
        assert_eq!(
            selector.hardware.architectures,
            vec![CpuArchitecture::X86_64]
        );
    }

    #[test]
    fn selector_json_hardware_is_the_fallback_when_the_column_is_empty() {
        let selector = VirtualPoolSelector::from_pool_columns(
            &serde_json::Value::Null,
            &serde_json::json!({ "strategy": "manual-ui", "hardware": { "min_cpu_logical": 4 } }),
        )
        .expect("legacy selector parses");
        assert_eq!(selector.hardware.min_cpu_logical, Some(4));

        let unconstrained = VirtualPoolSelector::from_pool_columns(
            &serde_json::Value::Null,
            &serde_json::json!({ "strategy": "manual-ui" }),
        )
        .expect("legacy selector without hardware parses");
        assert!(unconstrained.is_unconstrained());
    }

    #[test]
    fn malformed_hardware_selector_column_is_refused() {
        VirtualPoolSelector::from_pool_columns(
            &serde_json::json!({ "hardware": { "min_cpu_logical": "four" } }),
            &serde_json::json!({}),
        )
        .expect_err("a typo in the column must not fall through to selector_json");
    }

    #[test]
    fn placement_policy_defaults_to_least_loaded() {
        for value in [serde_json::Value::Null, serde_json::json!({})] {
            let policy = VirtualPoolPlacementPolicy::from_placement_policy_json(&value)
                .expect("default policy parses");
            assert_eq!(policy.strategy, PoolPlacementStrategy::LeastLoaded);
        }
    }

    #[test]
    fn placement_policy_parses_known_strategies() {
        let policy = VirtualPoolPlacementPolicy::from_placement_policy_json(
            &serde_json::json!({ "strategy": "most-memory" }),
        )
        .expect("most-memory parses");
        assert_eq!(policy.strategy, PoolPlacementStrategy::MostMemory);
        assert_eq!(policy.strategy.as_str(), "most-memory");
    }

    #[test]
    fn placement_policy_refuses_unknown_strategy() {
        VirtualPoolPlacementPolicy::from_placement_policy_json(
            &serde_json::json!({ "strategy": "round-robin" }),
        )
        .expect_err("unknown strategy must not fall back to least-loaded");
    }
}
