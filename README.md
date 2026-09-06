# Pion

[![CI](https://github.com/unified-field-dev/pion/actions/workflows/ci.yml/badge.svg)](https://github.com/unified-field-dev/pion/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[GitHub](https://github.com/unified-field-dev/pion) · `cargo doc -p pion --open`

## About

Pion is a **cell control-plane** library and slim server for fleet agents. It
persists node/cell inventory from heartbeats, runs a leased node-action queue,
enrolls hosts with one-time tickets, and delivers signed agent handoff directives
over a stable `/api/parton/*` HTTP surface. Storage is Valence: SQLite for an
embedded control plane in your host application, or distributed Postgres with an
in-memory cache.

- **Heartbeat ingest** — node/cell inventory, observed status, container observations
- **Node-action queue** — enqueue, claim, lease, report, reconcile; default-deny capabilities
- **Host enrollment** — one-time tickets; source-IP verify by default
- **Handoff directives** — mint and deliver signed Parton directives on heartbeat
- **Auth** — fail-closed shared token or SPIFFE; rate/body limits; per-node binding
- **Secrets** — Neutrino `$secret_ref` placeholders in the queue; values resolve only on claim
- **Deploy helpers / projections** — operator read models and deploy preflight helpers
- **Metrics** — Spectra helpers + Prometheus; embeddable `parton_router` / `pion-server`

Orchestrators and fleet composers enqueue actions in-process. Optional peers plug
in via process-local hooks (`bootstrap_notify`, `registry_storage_notify`) without
a `pion → orchestrator` compile dependency. The Valence logical partition name
`gluon` is a historical opaque wire key.

See [`pion/README.md`](pion/README.md) for crate-level API details.

## Examples

See [`pion/examples/README.md`](pion/examples/README.md) for the teaching path
(`parton_router_smoke` and `pion-server`).

## Getting started

```bash
export CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR=target-pion
cargo test -p pion --features runtime
```

```bash
export PARTON_SHARED_TOKEN="dev-shared-token"
export CARGO_TARGET_DIR=target-pion
cargo run -p pion-server
```

## Security

Vulnerability reporting: [`SECURITY.md`](SECURITY.md). Dependency policy:
[`docs/supply-chain.md`](docs/supply-chain.md). SPIFFE auth modes:
[`docs/spiffe.md`](docs/spiffe.md).

## Verify

CI runs on every push and PR ([`.github/workflows/ci.yml`](.github/workflows/ci.yml)):

```bash
export CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR=target-pion
cargo fmt --all --check
cargo check -p pion --features runtime
cargo check -p pion-server
cargo clippy -p pion --all-targets --features runtime -- -D warnings
cargo clippy -p pion-server --all-targets -- -D warnings
cargo test -p pion --features runtime
cargo deny check
```

Full command block: [`VERIFICATION.md`](VERIFICATION.md) (details in
[`docs/VERIFICATION.md`](docs/VERIFICATION.md)). Contribute:
[`CONTRIBUTING.md`](CONTRIBUTING.md).

## FAQ

**Is it production-ready?** v0.1.0. Heartbeat ingest, node-action queue, enrollment,
fail-closed agent auth, rate limits, secret-ref indirection, and optional SPIFFE
verification are implemented and tested. It is currently experimental.

**Do I need Postgres?** No. SQLite (`VALENCE_SQLITE_PATH`) embeds the control plane
in your host process. For a shared fleet database, set `PION_STORAGE=hybrid` and
`DATABASE_URL` for distributed Postgres with an in-memory cache (`db-hybrid`).

**How do agents authenticate to Pion?** By default a shared token (`PARTON_SHARED_TOKEN`)
on every `/api/parton/*` route, plus `x-parton-node-id` binding. `pion-server` refuses
to start without the token unless `PION_ALLOW_INSECURE=1` (testing only). For production
fleets, enable SPIFFE (`PION_AUTH_MODE=dual` then `spiffe`) — see
[`docs/spiffe.md`](docs/spiffe.md).

**Where do secrets in queued actions come from?** Queued actions store Neutrino
`$secret_ref` placeholders, not secret values. Values resolve only when an authenticated
agent claims the action, and are not written back into the queue.

**Where does Parton fit in?** Parton is the node agent on the other side of the HTTP
surface Pion serves — see [github.com/unified-field-dev/parton](https://github.com/unified-field-dev/parton).

**Where do orchestrators fit in?** A fleet composer or other orchestrator enqueues
node actions and may register process-local hooks.

## License

MIT. See [LICENSE](LICENSE).
