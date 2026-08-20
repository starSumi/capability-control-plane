import { cp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  humanPages,
  projectionDocuments,
} from "../src/content-catalog.mjs";

const siteRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repositoryRoot = path.resolve(siteRoot, "..");
const generatedRoot = path.join(siteRoot, ".generated");
const outputRoot = path.join(generatedRoot, "public", "_mcp");
const humanRoot = path.join(generatedRoot, "content", "docs");
const portablePath = /^[A-Za-z0-9_/-]+\.mdx?$/u;

function humanRoute(slug) {
  return slug === "index" ? "/" : `/${slug}/`;
}

function assertContained(root, candidate, label) {
  const relative = path.relative(root, candidate);
  if (relative.startsWith("..") || path.isAbsolute(relative)) {
    throw new Error(`${label} escapes its declared root.`);
  }
}

function renderHumanDocument(source, entry) {
  const normalized = source.replaceAll("\r\n", "\n");
  const match = /^(?:\uFEFF)?# ([^\n]+)\n(?:\n)?/u.exec(normalized);
  if (!match) {
    throw new Error(`Human source '${entry.source}' must start with one H1.`);
  }
  if (match[1].trim() !== entry.title) {
    throw new Error(
      `Human source '${entry.source}' H1 does not match catalog title '${entry.title}'.`,
    );
  }
  const frontmatter = [
    "---",
    `title: ${JSON.stringify(entry.title)}`,
    `description: ${JSON.stringify(entry.description)}`,
    "editUrl: false",
    "---",
    "",
  ].join("\n");
  return `${frontmatter}${normalized.slice(match[0].length)}`;
}

await rm(generatedRoot, { recursive: true, force: true });
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

for (const entry of humanPages) {
  const source = path.resolve(repositoryRoot, entry.source);
  const target = path.resolve(humanRoot, entry.source);
  assertContained(repositoryRoot, source, "Human source");
  assertContained(humanRoot, target, "Human target");
  const content = await readFile(source, "utf8");
  await mkdir(path.dirname(target), { recursive: true });
  await writeFile(target, renderHumanDocument(content, entry), "utf8");
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

const routes = Object.fromEntries(
  humanPages.map(({ source, slug }) => [source, humanRoute(slug)]),
);
await writeFile(
  path.join(outputRoot, "sumi-docs-routes.json"),
  `${JSON.stringify({ version: 1, routes }, null, 2)}\n`,
  "utf8",
);
