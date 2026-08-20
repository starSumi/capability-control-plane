# Hardening portfolio

The selected hardening strategy is a small authority kernel with explicit
boundaries. It is intentionally not an HTTP proxy, hook interceptor, event bus,
or MCP-everything gateway.

## Accepted controls

- strict `serde` decoding with unknown-field denial;
- typed authority and permission sets;
- compiled ceilings that policy cannot widen;
- pure engine with no I/O or execution dependency;
- rooted, read-only, indirect-path-denying JSON store with stable-handle reads;
- desired/observed version, policy digest, and per-item generation/digest CAS;
- structured denial codes and no raw query echo;
- lockfile, source/license/advisory policy, and SHA-pinned CI actions;
- threat model, security policy, and reproducible benchmark contract.

## Deferred controls

Sandboxing, eBPF/Landlock, mTLS, distributed leases, keyless/HSM signing,
model anomaly detection, AIBOM, and formal verification are not free security
badges. They require a real execution or deployment boundary, an owner, a
workload, and a recovery/evidence contract first.

## Implementation order

1. Keep current tests and deny policy green.
2. Add fuzz/property coverage and advisory database checks.
3. Add a read-only shadow event/replay artifact with retention and redaction.
4. Introduce one provider adapter only after authority and approval contracts
   are pinned.
5. Re-run threat modeling before any write, execute, network, or client hook.
