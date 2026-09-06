# pion-testkit

Shared matrix/scenario/bootstrap harness for `pion-e2e` and `pion-bench`. Installs Valence
(SQLite or Hybrid) plus an in-process Axum `pion::runtime::parton_router` for a `MatrixSpec` row
(`BootstrapSession`), then drives multi-step correctness/latency scenarios against it
(`ScenarioRunner`).

## Modules

| Module | Role |
|--------|------|
| `matrix` | `MatrixSpec` — topology / executor / backend / hardware axes for a benchmark row |
| `bootstrap` | `BootstrapSession` — stands up DB + control-plane router for one matrix row |
| `scenario` | `ScenarioSpec` / `ScenarioStep` — declarative multi-step flows |
| `runner` | `ScenarioRunner` — executes a `ScenarioSpec` against a `BootstrapSession` |
| `bench_image` | Resolves the container image ref/digest used by deploy-shaped scenario steps |
| `remote_parton` | HTTP client for a remote (out-of-process) Parton agent — see below |

## Remote Parton client (`remote_parton`)

`ScenarioRunner`'s `Executor::DockerCli` path executes container actions **in-process** via
Parton's `DockerCliActionExecutor` — fine for same-host topologies, but it can't reach a
separately deployed agent host over the network.

`remote_parton::RemoteParton` is the client-side building block for that gap: it POSTs a
`parton::ContainerActionRequest` to `{base_url}/agent/execute` on a remote host and decodes the
`parton::ContainerActionResponse`, optionally sending an `x-parton-token` shared-secret header.

```rust,no_run
use pion_testkit::RemoteParton;

# async fn demo(request: &parton::ContainerActionRequest) -> anyhow::Result<()> {
if let Some(client) = RemoteParton::from_env()? {
    let response = client.execute_action(request).await?;
    println!("remote agent {} -> success={}", client.base_url(), response.success);
}
# Ok(())
# }
```

Configuration (both optional; absence of the URL means "remote mode disabled"):

| Env var | Purpose |
|---------|---------|
| `PION_REMOTE_PARTON_URL` | Base URL of the remote agent, e.g. `http://10.0.1.9:8080`. Unset ⇒ `RemoteParton::from_env()` returns `Ok(None)`. |
| `PION_REMOTE_PARTON_TOKEN` | Optional shared token sent as `x-parton-token` on every request. |

**Status: client + contract test only.** Parton is pull-based today (agents poll
`/actions/claim` / `/actions/result` / `/actions/extend-lease`); there is no `POST /agent/execute`
handler on the real `parton` binary yet, and `ScenarioRunner` does not have a remote-agent
`Executor` variant wired into its per-step dispatch. `remote_parton`'s own tests stand up an
in-process Axum server that plays the role of that not-yet-built endpoint to validate the
client's request/response contract, auth header, and failure handling. Treat this module as the
foundation for a future remote-agent runner mode, or as a starting point if implementing the real
`/agent/execute` handler on the Parton side.

## Related

- Benchmarks: [`pion-bench`](../pion-bench/README.md)
- Correctness: [`pion-e2e`](../pion-e2e)
