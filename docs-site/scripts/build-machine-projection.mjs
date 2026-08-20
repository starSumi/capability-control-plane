import { cp, mkdir, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { projectionDocuments } from "../src/content-catalog.mjs";

const siteRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repositoryRoot = path.resolve(siteRoot, "..");
const outputRoot = path.join(siteRoot, ".generated", "_mcp");
const portablePath = /^[A-Za-z0-9_/-]+\.mdx?$/u;

function assertContained(root, candidate, label) {
  const relative = path.relative(root, candidate);
  if (relative.startsWith("..") || path.isAbsolute(relative)) {
    throw new Error(`${label} escapes its declared root.`);
  }
}

await rm(path.join(siteRoot, ".generated"), { recursive: true, force: true });
await mkdir(outputRoot, { recursive: true });

for (const entry of projectionDocuments) {
  if (!portablePath.test(entry.target) || entry.target.includes("//")) {
    throw new Error(`Invalid machine projection target: ${entry.target}`);
  }
  const source = path.resolve(repositoryRoot, entry.source);
  const target = path.resolve(outputRoot, entry.target);
  assertContained(repositoryRoot, source, "Projection source");
  assertContained(outputRoot, target, "Projection target");
  await mkdir(path.dirname(target), { recursive: true });
  await cp(source, target, { force: true, errorOnExist: false });
}

const manifest = {
  version: 1,
  documents: projectionDocuments.map(({ target }) => target),
};

await writeFile(
  path.join(outputRoot, "sumi-docs-manifest.json"),
  `${JSON.stringify(manifest, null, 2)}\n`,
  "utf8",
);
