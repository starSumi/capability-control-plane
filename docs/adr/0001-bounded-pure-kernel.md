# ADR 0001: Start with a bounded pure kernel

- Status: accepted
- Date: 2026-08-20
- Scope: V1 repository bootstrap

## Context

The input proposals correctly identify context as a scarce working set and
reconciliation as a useful level-triggered abstraction. They also overreach
into an always-on MCP gateway, proxy, hook interceptor, database, and universal
scheduler without a measured multi-writer or execution requirement.

## Decision

Build three crates: protocol, pure engine, and read-only CLI. Keep desired state
outside the engine, use opaque provider authority, make ambiguity/no-match
explicit, and emit plans with generation/digest preconditions. Do not execute,
persist, proxy, or mutate a client in V1.

## Consequences

The first release is small, deterministic, easy to benchmark and audit. It does
not yet load a real SKILL.md or replace Codex native surfaces. Integration must
earn its way in through a provider contract and shadow evidence.

