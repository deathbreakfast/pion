//! Operator undrain: move a `draining` control-plane node back to `online`.

use anyhow::{Context, Result};
use chrono::Utc;
use valence::{Model, Valence};

use crate::generated::{PionControlPlaneNode, PionControlPlaneNodeStatus};

/// Moves a `draining` node back to `online` and refreshes its `updated_at`.
///
/// Runs under the actor on `v`. The update needs `OperateCloudResources` (see
/// [`crate::PION_OPERATE_CLOUD_RESOURCES`]) unless the caller is System; this function never
/// elevates. Returns `Ok(false)` without writing when the node is missing or isn't draining.
///
/// # Stale status only
///
/// Heartbeats recompute `status` from container health ([`crate::map_health_to_node_status`]
/// maps Degraded to `draining`). If the node still reports exited containers, its next heartbeat
/// sets it back to `draining`, so this only helps when Pion's stored status is stale.
///
/// # Errors
///
/// Returns `Err` when the node can't be loaded, or when the update is denied (the caller lacks
/// the permission) or fails to commit.
pub async fn undrain_node(v: &Valence, node_id: &str) -> Result<bool> {
    let Some(node) = PionControlPlaneNode::get(node_id, v, valence::use_!("When an operator **undrains a node**, we **load the node** to check that it is still marked draining. Only a draining node is moved back to online; anything else is left as it is."))
        .await
        .with_context(|| format!("load node {node_id} for undrain"))?
    else {
        return Ok(false);
    };
    if *node.status() != PionControlPlaneNodeStatus::Draining {
        return Ok(false);
    }

    node.get_mutable(v, valence::use_!("When an operator **undrains a node**, we **set its status back to online** and stamp the change time. Placement and deploys can pick the node again."))
        .set_status(PionControlPlaneNodeStatus::Online)?
        .set_updated_at(Utc::now())?
        .commit()
        .await
        .with_context(|| format!("commit undrain of node {node_id}"))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::{DateTime, Duration};
    use valence::Actor;

    use super::*;
    use crate::generated::PionControlPlaneNodeConnectionMode;
    use crate::privacy_policies::{set_permission_checker, PermissionChecker};

    const OPERATOR: &str = "operator";

    /// Grants `OperateCloudResources` to [`OPERATOR`] only.
    struct OperatorOnly;

    #[async_trait]
    impl PermissionChecker for OperatorOnly {
        async fn actor_can(&self, v: &Valence, permission_name: &str) -> anyhow::Result<bool> {
            Ok(permission_name == "OperateCloudResources"
                && matches!(v.actor(), Actor::User { user_id } if user_id == OPERATOR))
        }
    }

    async fn test_valence() -> Valence {
        // Every test installs the same checker; later installs are rejected harmlessly.
        let _ = set_permission_checker(Arc::new(OperatorOnly));
        crate::valence_bootstrap::bootstrap_sqlite_memory()
            .await
            .expect("sqlite memory bootstrap")
            .valence("undrain-test")
            .expect("valence build")
    }

    fn as_user(v: &Valence, user_id: &str) -> Valence {
        v.with_actor(Actor::User {
            user_id: user_id.to_string(),
        })
    }

    fn an_hour_ago() -> DateTime<Utc> {
        Utc::now() - Duration::hours(1)
    }

    async fn seed_node(v: &Valence, id: &str, status: PionControlPlaneNodeStatus) {
        let at = an_hour_ago();
        let node = PionControlPlaneNode::new(
            "local-default".to_string(),
            format!("{id}.local"),
            PionControlPlaneNodeConnectionMode::Agent,
            status,
            String::new(),
            serde_json::json!({}),
            serde_json::json!({}),
            at,
            at,
        )
        .expect("build node");
        PionControlPlaneNode::upsert(id, node, v, valence::use_!("**Test:** the undrain suite **seeds a node** with a chosen status so it can check which nodes undrain moves to online. CI and developers running the suite only."))
            .await
            .expect("seed node");
    }

    async fn load(v: &Valence, id: &str) -> PionControlPlaneNode {
        PionControlPlaneNode::get(id, v, valence::use_!("**Test:** the undrain suite **reads a node back** to check its status and change time after undrain. CI and developers running the suite only."))
            .await
            .expect("load node")
            .expect("node exists")
    }

    #[tokio::test]
    async fn operator_moves_draining_node_online_once() {
        let v = test_valence().await;
        seed_node(&v, "n1", PionControlPlaneNodeStatus::Draining).await;
        let before = *load(&v, "n1").await.updated_at();
        let op = as_user(&v, OPERATOR);

        assert!(undrain_node(&op, "n1").await.expect("undrain"));
        let node = load(&v, "n1").await;
        assert_eq!(*node.status(), PionControlPlaneNodeStatus::Online);
        assert!(*node.updated_at() > before);

        assert!(!undrain_node(&op, "n1").await.expect("second undrain"));
    }

    #[tokio::test]
    async fn non_draining_and_missing_nodes_are_left_alone() {
        let v = test_valence().await;
        let op = as_user(&v, OPERATOR);
        for (id, status) in [
            ("online", PionControlPlaneNodeStatus::Online),
            ("offline", PionControlPlaneNodeStatus::Offline),
            ("failed", PionControlPlaneNodeStatus::Failed),
        ] {
            seed_node(&v, id, status.clone()).await;
            let before = *load(&v, id).await.updated_at();
            assert!(!undrain_node(&op, id).await.expect("undrain"), "{id}");
            let node = load(&v, id).await;
            assert_eq!(*node.status(), status, "{id}");
            assert_eq!(*node.updated_at(), before, "{id}");
        }
        assert!(!undrain_node(&op, "missing").await.expect("undrain missing"));
    }

    #[tokio::test]
    async fn user_without_permission_is_denied_and_row_unchanged() {
        let v = test_valence().await;
        seed_node(&v, "n1", PionControlPlaneNodeStatus::Draining).await;
        let before = *load(&v, "n1").await.updated_at();

        assert!(undrain_node(&as_user(&v, "viewer"), "n1").await.is_err());
        let node = load(&v, "n1").await;
        assert_eq!(*node.status(), PionControlPlaneNodeStatus::Draining);
        assert_eq!(*node.updated_at(), before);
    }

    #[tokio::test]
    async fn system_can_undrain() {
        let v = test_valence().await;
        seed_node(&v, "n1", PionControlPlaneNodeStatus::Draining).await;

        assert!(undrain_node(&v, "n1").await.expect("undrain"));
        assert_eq!(
            *load(&v, "n1").await.status(),
            PionControlPlaneNodeStatus::Online
        );
    }
}
