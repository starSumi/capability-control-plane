# Lifecycle

The resource lifecycle is explicit and restartable:

```text
author Rust semantics -> generate/check projections -> validate -> admit
       -> index -> route/plan -> provider assemble (future)
       -> execute (outside V1) -> observe -> evaluate -> evolve
```

## States

- **Draft**: a local change with no compatibility promise.
- **Validated**: strict decoding, admission, tests, and schema checks pass.
- **Shadow**: route/plan observations are recorded without changing a client.
- **Promoted**: a separately owned adapter has met its evidence gate.
- **Deprecated**: a version remains readable for its compatibility window but
  is no longer selected by default.
- **Rolled back**: desired policy or adapter version is changed and reconciled;
  rollback is not reverse replay of side effects.

V1 implements Draft/Validated and can emit a plan suitable for a future Shadow
caller. `capctl shadow` provides a non-mutating, provenance-bound replay surface
for a seed held-out fixture; it does not persist shadow state or perform
promotion.

Protocol changes are authored once in `capability-protocol`. Generated Schema
and client bindings are reproducible projections and must be current before a
change can enter Validated state. Breaking semantics require a new explicit API
version; generated-file edits never establish compatibility.

## Generation and recovery

Every desired resource has a monotonic generation. Every plan carries desired
and observed resource versions, catalog digest, policy identity/generation/
digest, and expected observed generation/digest for updates. A create action
requires observed absence. An actuator must re-read and verify every
precondition before acting, then re-observe the result. If it crashes, in-memory
work is discarded and the next invocation recomputes from desired plus actual
state. Missing or corrupt derived status is quarantined/rebuilt; desired intent
is never inferred from logs.

## Promotion contract

Promotion is not a score in this repository. A future adapter must provide:

1. held-out fixtures with ground-truth provenance;
2. selector version, catalog digest, and policy generation per observation;
3. explicit actual-use evidence from the native owner, not caller-supplied
   claims;
4. false-positive, no-match, ambiguity, p95 latency, and context-cost slices;
5. rollback trigger, compatibility window, retention policy, and owner.

The replay evaluator uses `GroundTruthSource::HeldOutReview` only for reviewed
fixture cases. `NativeInvocation` and `ProviderRead` are reserved for future
owner-authorized adapters and must not be asserted from caller-supplied actuals.
