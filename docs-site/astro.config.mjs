import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import { sitePages } from "./src/content-catalog.mjs";

function normalizeBasePath(value) {
  if (!value) return "/";
  if (!value.startsWith("/") || value.includes("\\") || value.includes("..")) {
    throw new Error("BASE_PATH must be a root-relative path without traversal.");
  }
  return value.endsWith("/") ? value : `${value}/`;
}

const base = normalizeBasePath(process.env.BASE_PATH);
const site = process.env.SITE_URL;

export default defineConfig({
  ...(site ? { site } : {}),
  base,
  publicDir: ".generated/public",
  integrations: [
    starlight({
      title: "Capability Control Plane",
      description:
        "Contracts, evidence, security boundaries, and measured evolution of the capability control plane.",
      social: [
        {
          icon: "github",
          label: "Sumi Docs upstream",
          href: "https://github.com/starSumi/sumi-docs",
        },
      ],
      sidebar: [
        {
          label: "System",
          items: sitePages.map(({ label, slug }) => ({ label, slug })),
        },
      ],
    }),
  ],
});
