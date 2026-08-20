# ADR 0002: Provider authority stays opaque

- Status: accepted
- Date: 2026-08-20

Capability descriptors carry `provider`, `package`, and `resource`, not local
paths, URLs, shell snippets, or credentials. The engine can validate and rank
the identity but cannot dereference it. This preserves owner/version/permission
boundaries when a future Codex, Claude, MCP, or local provider adapter is added.

