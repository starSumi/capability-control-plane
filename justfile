set shell := ["pwsh", "-NoLogo", "-NoProfile", "-Command"]

default:
    @just --list

fmt:
    cargo fmt --all -- --check

check:
    cargo check --workspace --all-targets --locked

codegen:
    cargo run -p xtask --locked -- codegen

codegen-check:
    cargo run -p xtask --locked -- codegen --check

clippy:
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

test:
    cargo nextest run --workspace --all-features --locked

test-cargo:
    cargo test --workspace --all-features --locked

deny:
    cargo deny check

audit:
    cargo audit

publication-guard:
    bash scripts/publication-guard.sh HEAD

install-git-guards:
    git config --local core.hooksPath .githooks
    git config --local user.useConfigOnly true

bench-smoke:
    cargo bench -p capability-engine --bench routing -- --test

bench:
    cargo bench -p capability-engine --bench routing

ci: fmt check codegen-check clippy test-cargo
