//! Valence `PolicyEvaluator` rules that gate Pion writes behind named permissions.
//!
//! Pion is an L1 host kit and does not depend on Gauge (L2). A [`PermissionGatePolicy`] asks the
//! process-wide [`PermissionChecker`] the host installs with [`set_permission_checker`], typically
//! one that calls `gauge::service::actor_can`. System actors always pass. Every other actor is
//! denied until a checker is installed, so a host that forgets the wiring fails closed.

use std::any::Any;
use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use valence::{Actor, ActorContext, Error, PolicyEvaluator, PrivacyOperation, Result, Valence};

/// Answers whether the actor on a Valence handle holds a named permission.
#[async_trait]
pub trait PermissionChecker: Send + Sync {
    /// Returns `Ok(true)` when the actor on `v` holds `permission_name`.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the permission store can't be read.
    async fn actor_can(&self, v: &Valence, permission_name: &str) -> anyhow::Result<bool>;
}

static PERMISSION_CHECKER: OnceLock<Arc<dyn PermissionChecker>> = OnceLock::new();

/// One-time install (typically from the host binary at startup).
///
/// # Errors
///
/// Returns `Err` with the rejected checker when one was already installed.
pub fn set_permission_checker(
    checker: Arc<dyn PermissionChecker>,
) -> std::result::Result<(), Arc<dyn PermissionChecker>> {
    PERMISSION_CHECKER.set(checker)
}

/// Returns the checker installed by [`set_permission_checker`], if any.
pub fn permission_checker() -> Option<Arc<dyn PermissionChecker>> {
    PERMISSION_CHECKER.get().cloned()
}

/// A Valence privacy rule that requires the actor to hold a specific permission.
#[derive(Debug, Clone)]
pub struct PermissionGatePolicy {
    /// Valence privacy rule name (namespaced, e.g. `"pion::OPERATE_CLOUD_RESOURCES"`).
    pub rule_name: &'static str,
    /// Permission name passed to the installed [`PermissionChecker`].
    pub permission_name: &'static str,
}

fn actor_from_context(actor: &dyn ActorContext) -> Result<Actor> {
    serde_json::from_value(actor.actor_json().clone())
        .map_err(|e| Error::Internal(format!("invalid actor context: {e}")))
}

#[async_trait]
impl PolicyEvaluator for PermissionGatePolicy {
    fn name(&self) -> &'static str {
        self.rule_name
    }

    fn description(&self) -> Option<&'static str> {
        Some("Permission-backed policy rule")
    }

    async fn evaluate(
        &self,
        _op: PrivacyOperation,
        _record: &serde_json::Value,
        actor: &dyn ActorContext,
        v: &Valence,
    ) -> Result<bool> {
        let viewer = actor_from_context(actor)?;
        if viewer.is_system() {
            return Ok(true);
        }
        let Some(checker) = permission_checker() else {
            return Ok(false);
        };
        checker
            .actor_can(v, self.permission_name)
            .await
            .map_err(|e| {
                Error::Privacy(format!(
                    "Permission policy check failed for '{}': {}",
                    self.permission_name, e
                ))
            })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Requires `OperateCloudResources` to update control-plane node rows (for example undrain).
///
/// The permission name matches Gluon's `GLUON_OPERATE_CLOUD_RESOURCES`, so one Gauge grant
/// covers both crates.
pub const PION_OPERATE_CLOUD_RESOURCES: PermissionGatePolicy = PermissionGatePolicy {
    rule_name: "pion::OPERATE_CLOUD_RESOURCES",
    permission_name: "OperateCloudResources",
};
