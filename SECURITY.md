# Security Policy

## Supported versions

Security fixes are accepted against the latest `main` branch and tagged releases (`0.1.x`) of this repository.

## Reporting a vulnerability

Please **do not** open a public GitHub issue for security-sensitive reports.

Prefer one of the following:

1. **GitHub Security Advisories** — use [Report a vulnerability](https://github.com/unified-field-dev/pion/security/advisories/new) on this repository when available.
2. Contact the maintainers privately via the repository owner listed at https://github.com/unified-field-dev/pion.

Include:

- a description of the issue and its impact
- steps to reproduce or a proof of concept when possible
- affected crate names and versions

We will acknowledge receipt as soon as practical and coordinate a fix and disclosure timeline with you.

## Client certificates on the agent routes

Pion does not terminate TLS. Set `PION_REQUIRE_CLIENT_CERT=1` when the agent routes sit behind a proxy that enforces mutual TLS (for example HAProxy's `:8443` frontend with `verify required`). Pion then rejects any `/api/parton/*` request whose `x-ssl-client-verify` header is not `0`.

The header is only as trustworthy as the proxy. Have the mTLS frontend set it from `ssl_c_verify`, and delete any client-supplied copy on every other frontend. Without that, a caller on a public listener can send `x-ssl-client-verify: 0` themselves. Keep the flag off until the proxy does this.

## Scope

In scope: vulnerabilities in this repository's crates, documentation that could cause unsafe production defaults, and CI/supply-chain issues in this repository.

Out of scope: vulnerabilities solely in third-party databases or HTTP reverse proxies unless Pion mishandles them in a security-relevant way.
