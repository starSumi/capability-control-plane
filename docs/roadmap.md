# Roadmap

## V1: bounded pure kernel (current)

- [x] independent Git repository and Rust workspace;
- [x] strict protocol, policy admission, opaque authority;
- [x] deterministic sparse route with CJK normalization;
- [x] digest and generation-aware dry-run reconcile plan;
- [x] rooted read-only CLI, fixtures, tests, deny policy, CI skeleton;
- [x] threat model and hardening proposal portfolio.
- [x] Rust semantic SSOT with checked JSON Schema and TypeScript projections.
- [x] honest working-set contract: `N <= 10,000` bounded admission, with
  `topK` bounded output rather than an O(K) score workspace claim.
- [x] first reviewed root commit (`c830972`) with no remote or runtime
  authority; this is a reproducible baseline, not a release.

## V1.1: evidence before integration

- [ ] run hosted Linux/Windows/macOS CI on the committed baseline;
- [x] add byte-compared cross-platform route-golden fan-in (hosted execution
  remains pending);
- [ ] add larger held-out routing fixtures with provenance;
- [ ] add shadow event schema and replay evaluator, still non-mutating;
- [ ] calibrate thresholds from measurements rather than magic confidence;
- [ ] add property/fuzz and `cargo-llvm-cov` coverage artifact;
- [ ] obtain cargo audit advisory database in CI and publish SBOM/provenance.

## V2: one real provider adapter

- [ ] choose one owner-approved provider contract;
- [ ] implement read/search through provider authority, never ambient paths;
- [ ] add bounded context assembly with digest/version checks;
- [ ] define partial failure, retry, retention, privacy, and rollback semantics.

## Deferred until evidence

Database or daemon, event broker, HTTP proxy, MCP gateway, plugin execution,
automatic client injection, sub-agent scheduler, distributed leases, frontend,
hardware-aware allocation, signing service, and model-based semantic fallback.
