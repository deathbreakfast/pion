# SPIFFE / SPIRE workload identity

SPIFFE JWT-SVID **verification and agent presentation are implemented**. Standing
up SPIRE itself remains an operator concern.

## SPIFFE ID convention

```text
spiffe://{trust_domain}/parton/node/{node_id}
```

Example: `spiffe://example.org/parton/node/edge-1` for `PARTON_NODE_ID=edge-1`.

## Control plane (`PION_AUTH_MODE`)

| Value | Behavior |
|---|---|
| `shared_token` (default) | Shared-token / `x-parton-token` check |
| `dual` | Accept shared token **or** valid JWT-SVID |
| `spiffe` | Require JWT-SVID (`Authorization: Bearer …`) |

Required when using SPIFFE:

- `PION_SPIFFE_TRUST_DOMAIN` — trust domain string (no `spiffe://` prefix)
- `PION_SPIFFE_JWT_KEY_PATH` — PEM public key used to verify JWT-SVIDs (SPIRE JWT key), **or**
- `PION_SPIFFE_JWT_HMAC_SECRET` — HS256 secret for tests only
- `PION_SPIFFE_AUDIENCE` — optional; default `pion`

## Agent (`PARTON_AUTH_MODE`)

Same values as above. When `spiffe` or `dual`, the agent attaches a bearer JWT from (first wins):

1. `PARTON_SPIFFE_JWT`
2. `PARTON_SPIFFE_JWT_PATH`
3. `spire-agent api fetch jwt` using `SPIFFE_ENDPOINT_SOCKET` (override binary: `PARTON_SPIRE_AGENT_BIN`)

Audience defaults to `pion` (`PARTON_SPIFFE_AUDIENCE` / `PION_SPIFFE_AUDIENCE`).

## Rollout

1. Register agent workloads in SPIRE with the ID convention above.
2. Configure control-plane verify key + `PION_AUTH_MODE=dual`.
3. Configure agent `PARTON_AUTH_MODE=dual` (or `spiffe`) and workload API socket.
4. Flip the control plane to `PION_AUTH_MODE=spiffe` once all agents present SVIDs.
