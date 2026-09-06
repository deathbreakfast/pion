# pion-bench

Synthetic benchmarks for Pion control-plane + Parton agent flows (`BM-R*`, `BM-RF*`).

| Document | Role |
|----------|------|
| [`PERFORMANCE.md`](PERFORMANCE.md) | Research questions, scope, threats to validity |

Local JSON under `pion-bench/reports/` stays gitignored (see root `.gitignore`).

## Run (local)

From the pion repo root:

```bash
export CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-./target-pion-extract}"

# List experiments
cargo run -p pion-bench -- experiments

# BM-R0 — heartbeat RTT smoke (in-process stub)
cargo run -p pion-bench -- run --experiment bm-r0

# BM-R1 / BM-R2 — warm stub deploy / teardown latency
cargo run -p pion-bench -- run --experiment bm-r1 --ops 5
cargo run -p pion-bench -- run --experiment bm-r2 --ops 5

# BM-RF0 — firehose stub skeleton (N=10, c=2)
cargo run -p pion-bench -- run --experiment bm-rf0
```

Decision-grade numbers typically use a dedicated `c6i.large`-class host with a warm
process and fixed `CARGO_BUILD_JOBS`; see hardware notes in [`PERFORMANCE.md`](PERFORMANCE.md).

## Related

- Harness: [`pion-testkit`](../pion-testkit)
- Correctness: [`pion-e2e`](../pion-e2e/README.md)
