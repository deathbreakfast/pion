//! `LocalDocker` topology smoke — ignored unless `PION_E2E_DOCKER=1` and Docker is available.

use pion_testkit::{BootstrapSession, MatrixSpec, RunMode, ScenarioRunner, ScenarioSpec};

fn docker_e2e_enabled() -> bool {
    std::env::var("PION_E2E_DOCKER").is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

fn docker_available() -> bool {
    std::process::Command::new("docker")
        .args(["info"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Live docker-cli enroll → warm deploy → ready (shared nginx image).
#[tokio::test]
#[ignore = "set PION_E2E_DOCKER=1 and require a live Docker daemon"]
async fn local_docker_enroll_deploy_gate() -> anyhow::Result<()> {
    if !docker_e2e_enabled() {
        anyhow::bail!("PION_E2E_DOCKER not set to 1");
    }
    if !docker_available() {
        anyhow::bail!("docker info failed — daemon not available");
    }

    let mut session = BootstrapSession::new(MatrixSpec::local_docker())?;
    session.install().await?;
    let runner = ScenarioRunner::new(&session);
    let result = runner
        .run(
            &ScenarioSpec::enroll_deploy_warm_docker(),
            RunMode::Correctness,
        )
        .await?;
    assert!(result.error.is_none(), "{:?}", result.error);

    // Sad path: teardown → gone.
    let result_td = runner
        .run(
            &ScenarioSpec::enroll_teardown_gone_docker(),
            RunMode::Correctness,
        )
        .await?;
    assert!(result_td.error.is_none(), "{:?}", result_td.error);
    Ok(())
}
