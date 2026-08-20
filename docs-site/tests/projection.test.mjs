import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  humanPages,
  projectionDocuments,
  sitePages,
} from "../src/content-catalog.mjs";

const siteRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repositoryRoot = path.resolve(siteRoot, "..");
const generatedRoot = path.join(siteRoot, ".generated");

function canonicalBody(source) {
  const normalized = source.replaceAll("\r\n", "\n");
  const match = /^(?:\uFEFF)?# ([^\n]+)\n(?:\n)?/u.exec(normalized);
  assert.ok(match, "canonical human source must start with an H1");
  return normalized.slice(match[0].length);
}

function humanRoute(slug) {
  return slug === "index" ? "/" : `/${slug}/`;
}

test("site slugs and projection targets are unique and rooted", () => {
  const slugs = sitePages.map(({ slug }) => slug);
  const targets = projectionDocuments.map(({ target }) => target);

  assert.equal(new Set(slugs).size, slugs.length);
  assert.equal(new Set(targets).size, targets.length);
  for (const target of targets) {
    assert.match(target, /^[A-Za-z0-9_/-]+\.mdx?$/u);
    assert.equal(target.startsWith("/"), false);
    assert.equal(target.includes("\\"), false);
    assert.equal(target.split("/").includes(".."), false);
  }
});

test("generated human architecture preserves the canonical body without a duplicate H1", async () => {
  const source = await readFile(
    path.join(repositoryRoot, "docs", "architecture.md"),
    "utf8",
  );
  const generated = await readFile(
    path.join(generatedRoot, "content", "docs", "docs", "architecture.md"),
    "utf8",
  );
  const frontmatterEnd = generated.indexOf("---\n", 4);
  assert.notEqual(frontmatterEnd, -1);
  const body = generated.slice(frontmatterEnd + 4).replace(/^\n/u, "");

  assert.equal(body, canonicalBody(source));
  assert.equal((generated.match(/^# Architecture$/gmu) ?? []).length, 0);
  assert.match(generated, /^title: "Architecture"$/mu);
});

test("machine projection stays byte-identical and routes humans to Starlight", async () => {
  const rawSource = await readFile(
    path.join(repositoryRoot, "docs", "architecture.md"),
  );
  const rawProjection = await readFile(
    path.join(
      generatedRoot,
      "public",
      "_mcp",
      "docs",
      "architecture.md",
    ),
  );
  const routeMap = JSON.parse(
    await readFile(
      path.join(generatedRoot, "public", "_mcp", "sumi-docs-routes.json"),
      "utf8",
    ),
  );

  assert.deepEqual(rawProjection, rawSource);
  assert.equal(routeMap.routes["docs/architecture.md"], "/architecture/");
  assert.equal(routeMap.routes["SECURITY.md"], "/security/");
  assert.equal(routeMap.routes["README.md"], "/");
  assert.deepEqual(
    Object.keys(routeMap.routes).sort(),
    projectionDocuments.map(({ target }) => target).sort(),
  );
  for (const { source, slug } of humanPages) {
    assert.equal(routeMap.routes[source], humanRoute(slug));
  }
});
