# Security implementation phase 1

## Done

- strict versioned resources and denial codes;
- no unknown authorities, permissions, fields, or policy widening;
- bounded rooted JSON reader with symlink/reparse checks, stable-handle reads,
  and post-open path verification;
- pure route and reconcile algorithms;
- adversarial unit/property tests, fixed-width wire checks, query-redaction
  checks, policy-before-I/O ordering, and deterministic order checks;
- lockfile, `deny.toml`, Apache-2.0 baseline, and pinned Rust toolchain;
- local `cargo deny` and fresh `cargo audit` evidence;
- a reproducible development benchmark with explicit evidence gaps.

## Next gate

Add parser fuzzing, run the declared multi-platform CI on a commit, and record a
representative provider-level benchmark. No runtime adapter may merge until
those artifacts name an owner, scope, permissions, rollback, and actual provider
evidence.

A future actuator must not reuse the read-only store as its authority boundary.
It needs a platform-reviewed, handle-relative root implementation and must
enforce every plan CAS field against a fresh provider-owned observation.
