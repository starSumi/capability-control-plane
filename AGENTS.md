# Repository Agent Contract

This repository owns a generic open-source capability discovery and planning
kernel. It must not depend on personal paths, client loader roots, `.momo`, or
machine-local control-plane directories at runtime.

## Startup

1. Read `README.md`, `docs/architecture.md`, `docs/roadmap.md`, and the newest
   accepted ADR touching your scope.
2. Run `git status --short --branch` and preserve unrelated work.
3. Classify the change as protocol, pure engine, CLI boundary, security,
   benchmark, or documentation. Respect the ownership table below.
4. Do not call the project production-ready without the promotion evidence in
   `docs/testing.md`.

## Ownership

- `capability-protocol` is the only semantic SSOT for versioned wire
  compatibility, strict decoding, decisions, plans, and error envelopes.
- `capability-engine` owns deterministic algorithms and performs no I/O.
- `capctl` owns bounded local reads and structured output; it performs no writes.
- `xtask` owns mechanical projections from `capability-protocol`; it cannot
  introduce defaults, permissions, transitions, or compatibility semantics.
- `generated/` is checked output. Never edit a generated file directly.
- `docs-site` consumes canonical documents and generated clients; it does not
  own protocol semantics or runtime policy.
- `fixtures` prove contracts but do not define production policy.
- `docs/security` owns threat model and hardening decisions.
- GitHub workflows own CI only; they are not runtime architecture.

Dependencies flow `capctl -> capability-engine -> capability-protocol`,
`capctl -> capability-protocol`, and `xtask -> capability-protocol`. Reverse
edges, ambient path reads, execution code in the engine, provider-specific logic
in the protocol, and client-authored wire DTOs are forbidden.

## Required gates

Run, at minimum, the focused tests plus:

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo deny check
just publication-guard
```

Run `cargo audit` when advisory network access is available and
`just bench-smoke` for routing or normalization changes. A performance claim
requires the full benchmark evidence contract in `docs/benchmarks.md`.

## Security invariants

- Unknown fields, identities, authorities, permissions, and versions deny.
- Untrusted text can influence ranking, never permission or execution.
- Every locator remains opaque and provider-owned; do not add ambient paths.
- Plans bind desired/observed versions and policy identity/digest, and carry
  per-item generation/digest preconditions; status never proves a side effect
  without re-observation.
- No shell command construction from resource content.
- No network, process, plugin, hook, or client mutation in V1.
- Do not weaken a lint, policy, size bound, or CI pin without an ADR, test,
  measured reason, rollback, and explicit reviewer attention.

## Change discipline

Protocol changes require compatibility notes, regenerated projections,
`xtask codegen --check`, and client type-checking.
Algorithm changes require adversarial fixtures and deterministic order tests.
Supply-chain changes require `Cargo.lock`, `cargo deny`, and license review.
Never invent GitHub owners, CODEOWNERS identities, signing subjects, or release
repositories before those authorities exist.
