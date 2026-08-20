# Capability Control Plane Documentation Site

This directory is the independently buildable human and agent documentation
projection. The Rust workspace does not depend on it.

## Technology Decision

The site uses the technology and public compatibility surface proven by
[starSumi/sumi-docs](https://github.com/starSumi/sumi-docs):

- Astro and Starlight render the static human site.
- One reviewed catalog drives navigation, human source-to-route mapping, and the
  machine document list.
- A deterministic build step publishes strict Sumi Docs manifest v1 at
  `dist/_mcp/sumi-docs-manifest.json` together with raw canonical documents.
- The Sumi Docs MCP server remains an external, stateless, read-only consumer.

The evidence basis is upstream `main` commit
`adee6416e6c9376709d32bca615f4509da97200c`, recorded in
`upstream.lock.json`. At that commit, the upstream declares MIT licensing,
Node.js 25.5.0 or newer, pnpm 10.26.0, Astro 7.2.1, Starlight 0.41.7, a strict
manifest v1, an immutable manifest v2, and a separate MCP package. The checked
out upstream worktree was clean when verified on 2026-08-20.

This adapter deliberately implements the smallest compatible surface. It does
not import the unpublished upstream workspace packages, copy its MCP runtime,
or claim v2 conformance. Full v2 publication requires its schema,
canonicalization, content-addressed revisions, and conformance fixtures as one
accepted change; emitting a look-alike locator would weaken the contract.

Alternatives considered:

- `mdBook` was rejected because the selected reference product already defines
  Astro/Starlight and a human-plus-agent publication contract.
- Forking or vendoring the complete Sumi Docs monorepo was rejected because it
  would duplicate release ownership and pull MCP concerns into this repository.
- Consuming `@sumi-os/docs-mcp` from npm is deferred because upstream states no
  npm package or GitHub Release has yet been published for pre-release 0.1.0.

## Lifecycle And Ownership

Canonical product documents remain in the repository root and `docs/`.
`src/content-catalog.mjs` maps each reviewed source to a human route and the raw
machine projection. The build creates disposable Starlight inputs under
`.generated/content/` by adding catalog frontmatter and removing the source H1;
the remaining body is preserved. Raw source bytes and
`sumi-docs-routes.json` are published under `.generated/public/_mcp/`.
`.generated/` and `dist/` are derived state and are never authored directly.

The docs-site owner updates `upstream.lock.json` only after rechecking the
upstream remote, branch, commit, license, package manifests, build commands, and
compatibility tests. A changed upstream commit is a deliberate upgrade, not an
automatic floating dependency.

## Develop And Build

Prerequisites are Node.js 25.5.0 or newer and pnpm 10.26.0 through Corepack.

```powershell
cd docs-site
pnpm install --frozen-lockfile
pnpm run dev
```

The development site defaults to `http://127.0.0.1:4321`. Build and verify the
static site plus machine projection with:

```powershell
pnpm run verify
```

Set `SITE_URL` and `BASE_PATH` only for a deployment candidate. `BASE_PATH`
must be root-relative and traversal-free.

## Optional MCP Readback

Until Sumi Docs publishes a package or release, build its pinned source in a
separate checkout. From this repository root, point the external server at the
tracked adapter config:

```powershell
$sumiDocsCheckout = "<trusted-sumi-docs-checkout>"
pnpm --dir $sumiDocsCheckout install --frozen-lockfile
pnpm --dir $sumiDocsCheckout --filter @sumi-os/docs-mcp build
node "$sumiDocsCheckout/packages/mcp/dist/index.js" doctor --config docs-site/sumi-docs.config.json --json
node "$sumiDocsCheckout/packages/mcp/dist/index.js" serve --config docs-site/sumi-docs.config.json
```

For a deployed site, the same external server can consume the published
`_mcp/sumi-docs-manifest.json` over HTTPS. Neither mode grants mutation rights
to the repository or the Rust control-plane kernel.
