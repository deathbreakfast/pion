# Pion performance

Measured on AWS same-region (`c6i.large` hybrid docker-cli) and multi-region (WAN) hosts. Pion is the edge/ingest kit sitting above Parton and Photon for fleet-facing traffic paths. Fair-compare matrices live in the private `uf-live-cloud-lab` pion performance study.

## Latency and throughput

Warm deploy → Ready (BM-D1, n=10): p50/p95 **308 / 538 ms**. Same shared image on kind k8s: **949 / 1925 ms**. Teardown → Gone (BM-D2): **~174 / 184 ms**. Firehose starts (BM-DF0): about **2 starts/s** per agent host at the measured Tier A shape.

Same-region CP/agent layouts bound the baseline for edge ingest and fair-compare runs. WAN (multi-region) paths add RTT-dominated cost; quote WAN numbers only for products that truly span regions. Nomad warm-TTR rows from single-node `-dev` are not decision-grade.

## Guidance

Use same-region AWS figures for default capacity planning. Treat WAN deltas as additive latency budgets, not as a replacement for same-region peaks. Tier A did not validate 5k-vCPU / 200-region projections.

## How to read these results

AWS hardware tags are the capacity reference. Local IsolatedHarness runs are correctness/smoke only.
