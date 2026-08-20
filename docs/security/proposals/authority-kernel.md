# Hardening proposal: authority kernel

## Problem

A capability router is exposed to untrusted text, local resource documents,
policies, dependency metadata, and future provider adapters. A convenient
“unified” registry or proxy would blur discovery, authority, and execution.

## Proposal

Keep the engine pure and make authority opaque. Validate a versioned catalog
against a typed policy, rank only metadata, and emit a plan containing desired
generation, catalog digest, and expected observed generation. The CLI reads only
root-relative strict JSON and never writes or executes.

## Alternatives rejected

- HTTP shadow proxy: crosses native approval/authority boundaries and adds a
  new always-on failure surface.
- Static skill to MCP one-for-one: turns reusable context into live action and
  expands permissions without evidence.
- database/event bus in V1: creates a second truth and recovery burden before a
  measured multi-process need.
- fixed semantic confidence threshold: no local calibration or native score
  contract supports it.

## Risks and validation

The main residual risk is a future adapter violating the engine's assumptions.
Mitigate with provider-owned reads, digest/version checks, shadow replay,
adversarial fixtures, and a new ADR before any runtime side effect.

