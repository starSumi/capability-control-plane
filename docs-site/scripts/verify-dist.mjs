import { access, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { projectionDocuments, sitePages } from "../src/content-catalog.mjs";

const siteRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const distRoot = path.join(siteRoot, "dist");
const manifestPath = path.join(distRoot, "_mcp", "sumi-docs-manifest.json");
const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
const keys = Object.keys(manifest).sort();

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

for (const { slug } of sitePages) {
  const route = slug === "index" ? "index.html" : path.join(slug, "index.html");
  await access(path.join(distRoot, route));
}

console.log(
  JSON.stringify({
    ok: true,
    humanRoutes: sitePages.length,
    machineDocuments: manifest.documents.length,
  }),
);
