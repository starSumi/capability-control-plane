# ADR 0005: Defensive open-source baseline

- Status: accepted
- Date: 2026-08-20

The repository commits `Cargo.lock`, uses `deny.toml` to reject unknown
registries/Git sources and wildcard requirements, pins CI actions by SHA, runs
with read-only repository permissions, and documents security reporting without
inventing an endpoint or maintainer roster. Release signing and attestations
remain deferred until an actual remote owner and key lifecycle exist.

