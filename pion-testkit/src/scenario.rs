//! Declarative scenario steps shared by e2e (assert) and bench (measure).

use serde::{Deserialize, Serialize};

/// One step in an agent / control-plane scenario.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "step", rename_all = "snake_case")]
pub enum ScenarioStep {
    /// Create a host enrollment ticket and remember the wire token.
    EnrollAgent,
    /// Heartbeat until the node is accepted (stub: one successful ingest).
    HeartbeatUntilHealthy,
    /// `docker pull` the shared bench image (warm path; not counted in cold TTR).
    PrePullImage,
    /// Enqueue a `deploy_handoff` node action for the lab node.
    EnqueueDeploy,
    /// Claim the pending action and acknowledge without executing (schedule latency).
    ClaimAndAck,
    /// Claim the pending action and execute it (stub or docker-cli).
    ClaimAndExecute,
    /// Assert the target container is running (stub state or observation).
    AssertContainerRunning,
    /// Enqueue a `teardown_handoff` node action.
    EnqueueTeardown,
    /// Assert the target container is gone.
    AssertContainerGone,
    /// Concurrent deploy firehose (N actions, bounded concurrency).
    FirehoseDeploy {
        /// Number of deploy actions.
        n: u32,
        /// Max in-flight deploys.
        concurrency: u32,
    },
    /// Concurrent teardown firehose.
    FirehoseTeardown {
        /// Number of teardown actions.
        n: u32,
        /// Max in-flight teardowns.
        concurrency: u32,
    },
    /// Extend the lease on the current claimed command (stub OK).
    ExtendLease,
    /// Cancel a pending/running action (stub OK).
    CancelAction,
    /// Apply a `ReEnroll` handoff directive (stub OK).
    ApplyReEnroll,
    /// Apply a Revoke handoff directive (stub OK).
    ApplyRevoke,
}

/// Named scenario: stable id + ordered steps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioSpec {
    /// Stable scenario identifier.
    pub id: String,
    /// Ordered steps to execute.
    pub steps: Vec<ScenarioStep>,
}

impl ScenarioSpec {
    /// Enroll → heartbeat → `deploy_handoff` → claim → stub execute → success.
    ///
    /// Primary `IsolatedHarness` correctness smoke used by `pion-e2e` and BM-R1 warm-up.
    #[must_use]
    pub fn enroll_deploy_smoke() -> Self {
        Self {
            id: "enroll-deploy-smoke".into(),
            steps: vec![
                ScenarioStep::EnrollAgent,
                ScenarioStep::HeartbeatUntilHealthy,
                ScenarioStep::EnqueueDeploy,
                ScenarioStep::ClaimAndExecute,
                ScenarioStep::AssertContainerRunning,
            ],
        }
    }

    /// `LocalDocker` / AWS warm deploy with pre-pull (BM-D1 / e2e docker gate).
    #[must_use]
    pub fn enroll_deploy_warm_docker() -> Self {
        Self {
            id: "enroll-deploy-warm-docker".into(),
            steps: vec![
                ScenarioStep::EnrollAgent,
                ScenarioStep::HeartbeatUntilHealthy,
                ScenarioStep::PrePullImage,
                ScenarioStep::EnqueueDeploy,
                ScenarioStep::ClaimAndExecute,
                ScenarioStep::AssertContainerRunning,
            ],
        }
    }

    /// Warm deploy + teardown → gone (BM-D2).
    #[must_use]
    pub fn enroll_teardown_gone_docker() -> Self {
        Self {
            id: "enroll-teardown-gone-docker".into(),
            steps: vec![
                ScenarioStep::EnrollAgent,
                ScenarioStep::HeartbeatUntilHealthy,
                ScenarioStep::PrePullImage,
                ScenarioStep::EnqueueDeploy,
                ScenarioStep::ClaimAndExecute,
                ScenarioStep::AssertContainerRunning,
                ScenarioStep::EnqueueTeardown,
                ScenarioStep::ClaimAndExecute,
                ScenarioStep::AssertContainerGone,
            ],
        }
    }

    /// Cold pull + deploy (BM-D3): no `PrePullImage`; pull happens inside deploy if missing.
    #[must_use]
    pub fn enroll_deploy_cold_docker() -> Self {
        Self {
            id: "enroll-deploy-cold-docker".into(),
            steps: vec![
                ScenarioStep::EnrollAgent,
                ScenarioStep::HeartbeatUntilHealthy,
                ScenarioStep::EnqueueDeploy,
                ScenarioStep::ClaimAndExecute,
                ScenarioStep::AssertContainerRunning,
            ],
        }
    }

    /// Schedule path only: enqueue → claim ack (BM-D0).
    #[must_use]
    pub fn schedule_claim_smoke() -> Self {
        Self {
            id: "schedule-claim-smoke".into(),
            steps: vec![
                ScenarioStep::EnrollAgent,
                ScenarioStep::HeartbeatUntilHealthy,
                ScenarioStep::EnqueueDeploy,
                ScenarioStep::ClaimAndAck,
            ],
        }
    }

    /// Heartbeat RTT smoke (BM-R0): enroll once, then repeated healthy heartbeats.
    #[must_use]
    pub fn heartbeat_rtt_smoke() -> Self {
        Self {
            id: "heartbeat-rtt-smoke".into(),
            steps: vec![
                ScenarioStep::EnrollAgent,
                ScenarioStep::HeartbeatUntilHealthy,
            ],
        }
    }

    /// Single teardown after a warm stub deploy (BM-R2).
    #[must_use]
    pub fn enroll_teardown_smoke() -> Self {
        Self {
            id: "enroll-teardown-smoke".into(),
            steps: vec![
                ScenarioStep::EnrollAgent,
                ScenarioStep::HeartbeatUntilHealthy,
                ScenarioStep::EnqueueDeploy,
                ScenarioStep::ClaimAndExecute,
                ScenarioStep::AssertContainerRunning,
                ScenarioStep::EnqueueTeardown,
                ScenarioStep::ClaimAndExecute,
                ScenarioStep::AssertContainerGone,
            ],
        }
    }

    /// Small firehose stub (BM-RF0 skeleton): N=10, c=2.
    #[must_use]
    pub fn firehose_deploy_stub() -> Self {
        Self {
            id: "firehose-deploy-stub".into(),
            steps: vec![
                ScenarioStep::EnrollAgent,
                ScenarioStep::HeartbeatUntilHealthy,
                ScenarioStep::FirehoseDeploy {
                    n: 10,
                    concurrency: 2,
                },
            ],
        }
    }

    /// Edge firehose BM-DF0: N=50, c=4 with optional pre-pull for warm starts.
    #[must_use]
    pub fn firehose_deploy_df0() -> Self {
        Self {
            id: "firehose-deploy-df0".into(),
            steps: vec![
                ScenarioStep::EnrollAgent,
                ScenarioStep::HeartbeatUntilHealthy,
                ScenarioStep::PrePullImage,
                ScenarioStep::FirehoseDeploy {
                    n: 50,
                    concurrency: 4,
                },
            ],
        }
    }

    /// Configurable firehose (BM-DF1 / DM cells).
    #[must_use]
    pub fn firehose_deploy_n(n: u32, concurrency: u32) -> Self {
        Self {
            id: format!("firehose-deploy-n{n}-c{concurrency}"),
            steps: vec![
                ScenarioStep::EnrollAgent,
                ScenarioStep::HeartbeatUntilHealthy,
                ScenarioStep::PrePullImage,
                ScenarioStep::FirehoseDeploy { n, concurrency },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enroll_deploy_smoke_roundtrips_json() {
        let spec = ScenarioSpec::enroll_deploy_smoke();
        let json = serde_json::to_string(&spec).expect("serialize");
        let back: ScenarioSpec = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(spec, back);
    }
}
