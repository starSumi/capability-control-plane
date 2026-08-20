# ADR 0004: Rust kernel, frontend only after an operator workflow exists

- Status: accepted
- Date: 2026-08-20

Rust gives the protocol and engine explicit ownership, typed errors, low-level
resource control, and a mature supply-chain/tooling ecosystem. The CLI is the
first product surface because no real operator workflow has been demonstrated
for a dashboard. A future frontend may use Feature-Sliced Design, but it must
consume plans and status without owning policy or authority.

