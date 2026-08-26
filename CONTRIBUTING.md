# Contributing

Capability Control Plane is intentionally small. Contributions should remove a
measured failure mode or satisfy a real provider contract, not merely add
framework surface.

1. Open an issue describing workload, trust boundary, evidence, compatibility,
   and the smallest viable change.
2. Keep protocol, pure engine, and I/O boundary ownership separate.
3. Add tests that fail before the change, including no-match, ambiguity, stale
   generation, hostile input, or recovery behavior where relevant.
4. Run the gates in `AGENTS.md` and report the actual commands and results.
5. Update an ADR only for a durable architectural decision.

Commits should be reviewable and must not include secrets, local absolute paths,
generated benchmark noise, editor state, or unrelated formatting. Dependency
changes require license/source/advisory review and a committed lockfile.

Before the first public push, run `just install-git-guards`. The tracked
`pre-push` hook and `just publication-guard` audit author and committer identities
across the exact commits being published. GitHub noreply identities pass by
default. Add an address to `.github/public-emails.txt` only when the owner has
deliberately made it public; never disable provider privacy protection merely to
make a push pass.
