# ADR 0006: Rust protocol is the semantic SSOT

- Status: accepted
- Date: 2026-08-20
- Scope: cross-runtime contracts and client projections

## Context

Hand-authored Rust, JSON Schema, TypeScript, documentation, and client DTOs drift
independently. A nominal unified registry would not solve this if every consumer
could reinterpret permissions, defaults, versions, or state transitions.

## Decision

All cross-boundary semantics live as strict Rust types in
`capability-protocol`. The types derive serialization, JSON Schema, and
TypeScript representations. The crate also owns compiled ceilings, identifier
and digest grammar, resource kinds, fixed wire widths, and stable error codes;
the pure engine owns cross-resource policy application and algorithms. `xtask
codegen` materializes checked projections, and CI rejects any byte or file-set
drift.

Clients may map transport or host primitives but cannot define protocol
semantics. A semantic change starts in Rust, receives a compatibility decision,
and is then regenerated outward.

## Consequences

The protocol has one review point and clients receive deterministic artifacts.
The repository accepts a code-generation dependency and checked generated
files. Generated files are distribution surfaces, not authored truth.
