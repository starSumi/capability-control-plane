import { access, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  humanPages,
  projectionDocuments,
  sitePages,
} from "../src/content-catalog.mjs";

const siteRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repositoryRoot = path.resolve(siteRoot, "..");
const distRoot = path.join(siteRoot, "dist");
const manifestPath = path.join(distRoot, "_mcp", "sumi-docs-manifest.json");
const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
const keys = Object.keys(manifest).sort();

function humanRoute(slug) {
  return slug === "index" ? "/" : `/${slug}/`;
}

if (JSON.stringify(keys) !== JSON.stringify(["documents", "version"])) {
  throw new Error("Manifest v1 contains unknown or missing fields.");
}
if (manifest.version !== 1 || !Array.isArray(manifest.documents)) {
  throw new Error("Manifest v1 has an invalid version or document list.");
}

const expected = projectionDocuments.map(({ target }) => target);
if (JSON.stringify(manifest.documents) !== JSON.stringify(expected)) {
  throw new Error("Manifest document order drifted from the content catalog.");
}

for (const document of manifest.documents) {
  if (
    path.isAbsolute(document) ||
    document.includes("\\") ||
    document.split("/").includes("..")
  ) {
    throw new Error(`Manifest contains an unsafe path: ${document}`);
  }
  await access(path.join(distRoot, "_mcp", document));
}

const architectureSource = await readFile(
  path.join(repositoryRoot, "docs", "architecture.md"),
);
const architectureRaw = await readFile(
  path.join(distRoot, "_mcp", "docs", "architecture.md"),
);
if (!architectureRaw.equals(architectureSource)) {
  throw new Error("Raw architecture projection differs from its canonical source.");
}

const routeMap = JSON.parse(
  await readFile(path.join(distRoot, "_mcp", "sumi-docs-routes.json"), "utf8"),
);
if (
  routeMap.version !== 1 ||
  routeMap.routes["docs/architecture.md"] !== "/architecture/"
) {
  throw new Error("Architecture source is not mapped to its human route.");
}
if (
  JSON.stringify(Object.keys(routeMap.routes).sort()) !==
  JSON.stringify([...manifest.documents].sort())
) {
  throw new Error("Human route map does not cover the machine manifest exactly.");
}
for (const { source, slug } of humanPages) {
  if (routeMap.routes[source] !== humanRoute(slug)) {
    throw new Error(`Human route drifted for '${source}'.`);
  }
}

for (const { slug } of sitePages) {
  const route = slug === "index" ? "index.html" : path.join(slug, "index.html");
  const html = await readFile(path.join(distRoot, route), "utf8");
  if (/<a\b[^>]*\bhref=["'][^"']*\/_mcp\//iu.test(html)) {
    throw new Error(`Human route '/${slug}/' links readers to the raw MCP surface.`);
  }
}

const architectureHtml = await readFile(
  path.join(distRoot, "architecture", "index.html"),
  "utf8",
);
if (
  !/<h1\b[^>]*>Architecture<\/h1>/u.test(architectureHtml) ||
  !architectureHtml.includes('id="scope-and-authority"') ||
  !architectureHtml.includes("The control plane is a derived decision surface")
) {
  throw new Error("Human architecture route does not render canonical content.");
}

console.log(
  JSON.stringify({
    ok: true,
    humanRoutes: sitePages.length,
    machineDocuments: manifest.documents.length,
  }),
);
