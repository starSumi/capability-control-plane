# Benchmarks

Benchmarks measure the control plane, not an LLM leaderboard. HumanEval,
LiveCodeBench, SWE-bench Verified, ARC-AGI, GPQA, MATH, AIME, and GSM8K may
describe model capability or a coding-agent workload, but they do not prove
this repository's routing latency, memory behavior, authority isolation, or
recovery correctness.

## Current harness

`crates/capability-engine/benches/routing.rs` uses Divan and measures index build
and warm route for 32, 256, 412, 4096, and 10,000 descriptors with English, CJK,
and mixed queries. The ceiling case probes latency but does not by itself prove
allocation/RSS behavior or constant-memory Top-K. Run:

```powershell
just bench-smoke # one iteration per benchmark, fast regression check
just bench       # full local measurement
```

Every published result must include:

- Git commit and `Cargo.lock` digest;
- OS, CPU, memory, Rust toolchain, profile, and power/thermal state;
- fixture digest, descriptor distribution, query corpus provenance;
- cold vs warm definition, sample count, warmup, p50/p95/p99, and allocation;
- command, raw output, and whether the run was isolated from other builds.

There is no universal latency target yet. Targets become binding only after a
real adapter workload supplies a baseline and an owner accepts the SLO.

The current development-only Windows baseline is recorded in
[`benchmark-results/2026-08-20-windows-i5-1235u.md`](../benchmark-results/2026-08-20-windows-i5-1235u.md).
It is attached to a content snapshot because the repository has no first commit;
it must be rerun against a commit before it can support a release claim.
