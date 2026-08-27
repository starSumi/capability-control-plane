# Capability Control Plane

Capability Control Plane is a fail-closed, local-first kernel for discovering
large capability catalogs without placing every skill or tool description in an
agent's working context. It treats context as a scarce working set and returns
versioned decisions, never ambient authority.

The project is alpha. V1 can validate strict JSON resources, build a
deterministic-for-a-fixed-target weighted BM25-style index, route an intent, and compute a
side-effect-free reconciliation plan. It cannot execute tools, mutate clients,
install plugins, run as a daemon, or act as an MCP gateway.

## One-page model

```text
desired catalog + admission policy          observed snapshot
                 |                                 |
                 v                                 v
        [ strict protocol boundary ]      [ evidence, not truth ]
                 |                                 |
                 +----------+  +-------------------+
                            v  v
                    [ pure engine ]
                    /             \
          route decision       reconcile plan
          selected |             create | update
          ambiguous | noMatch    retain orphan
                    \             /
                     no execution
```

The physical analogy is deliberate but bounded:

- `spec` is potential: desired state that may cause work.
- `status` is observation: evidence that can be stale or rebuilt.
- admission and planning are force laws: deterministic constraints over state.
- authority is a boundary condition: a provider owns every resource address.
- `generation` and digest encode time: stale plans fail rather than overwrite.
- observations form feedback, but never become desired truth.

This borrows Kubernetes' level-triggered reconciliation, not Kubernetes-shaped
infrastructure. There is no API server, etcd, CRD, watch loop, or distributed
controller in V1.

## Try it

```powershell
cargo run -p capctl --locked -- validate `
  --root . `
  --catalog fixtures/catalog.v1alpha1.json `
  --policy fixtures/policy.v1alpha1.json

cargo run -p capctl --locked -- route `
  --root . `
  --catalog fixtures/catalog.v1alpha1.json `
  --policy fixtures/policy.v1alpha1.json `
  --query "resume session and recover conversation context"

cargo run -p capctl --locked -- reconcile `
  --root . `
  --desired fixtures/catalog.v1alpha1.json `
  --observed fixtures/observed.v1alpha1.json `
  --policy fixtures/policy.v1alpha1.json
```

Replay the seed held-out route corpus without executing or mutating anything:

```powershell
cargo run -p capctl --locked -- shadow `
  --root fixtures `
  --fixture shadow-heldout.v1alpha1.json `
  --catalog catalog.v1alpha1.json `
  --policy policy.v1alpha1.json
```

The replay output is privacy-safe and provenance-bound, but the checked-in
seven-case corpus is seed evidence rather than a production routing SLO.

Inputs must be root-relative strict JSON. Absolute paths, parent traversal,
symlinks/reparse points, unknown fields, unknown permissions, oversized
documents/catalog working sets, and untrusted authorities are denied. Route
results do not echo query text, matched terms, or a stable query digest.

## Repository map

- `crates/capability-protocol`: the only semantic SSOT for versioned wire types.
- `crates/capability-engine`: pure admission, retrieval, digest, and planning.
  Its admitted query workspace is bounded by `N <= 10,000`; `topK` bounds
  output and ranking, not the full score vector allocation.
- `crates/capctl`: read-only filesystem boundary and structured CLI.
- `crates/xtask`: deterministic Schema/TypeScript projection generator.
- `generated`: checked projections; never authored or interpreted independently.
- `docs-site`: Sumi Docs human and machine-readable documentation projection.
- `fixtures`: small executable contracts, not production policy.
- `docs`: architecture, lifecycle, security, benchmark, and decisions.
- `schemas`: schema lifecycle; `capctl schema` is the generated source.

Start with [the architecture](docs/architecture.md), then read
[security](docs/security/README.md), [testing](docs/testing.md), and the
[roadmap](docs/roadmap.md). Repository-specific agent instructions live in
[AGENTS.md](AGENTS.md).

## Build gates

```powershell
just fmt
just check
just codegen-check
just clippy
just test
just deny
just audit
just bench-smoke
pnpm --dir docs-site run verify
```

Rust `1.97.1` is pinned for reproducibility; the workspace declares MSRV
`1.88`. `Cargo.lock` is committed. CI actions are pinned by commit SHA and run
with read-only repository permission.

For local documentation development, install Node `25.5.0` and pnpm `10.26.0`,
then run `pnpm --dir docs-site dev`. The development site defaults to
`http://127.0.0.1:4321/`.

## Non-goals

- replacing Codex, Claude, ToolHive, or their native provider contracts;
- turning every static skill into an MCP tool;
- a universal semantic router with unmeasured confidence thresholds;
- a hidden HTTP proxy, hook, plugin injector, or master agent scheduler;
- claiming production readiness from unit tests or a single-machine benchmark.

See [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and
[GOVERNANCE.md](GOVERNANCE.md). Licensed under Apache-2.0.
