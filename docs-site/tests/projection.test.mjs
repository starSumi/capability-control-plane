import assert from "node:assert/strict";
import test from "node:test";
import { projectionDocuments, sitePages } from "../src/content-catalog.mjs";

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
