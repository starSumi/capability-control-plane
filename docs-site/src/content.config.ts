import { defineCollection } from "astro:content";
import { glob } from "astro/loaders";
import { docsSchema } from "@astrojs/starlight/schema";
import { humanPages } from "./content-catalog.mjs";

const siteRoot = new URL("../", import.meta.url);
interface HumanPageRoute {
  readonly source: string;
  readonly slug: string;
}
const humanPageRoutes = humanPages as readonly HumanPageRoute[];

function generateId({ entry }: { entry: string }): string {
  const prefix = ".generated/content/docs/";
  if (!entry.startsWith(prefix)) {
    throw new Error(`Unsupported documentation source '${entry}'.`);
  }
  const source = entry.slice(prefix.length);
  const document = humanPageRoutes.find(
    (candidate) => candidate.source === source,
  );
  if (!document) {
    throw new Error(`Generated documentation source '${source}' is not cataloged.`);
  }
  return document.slug;
}

export const collections = {
  docs: defineCollection({
    loader: glob({
      base: siteRoot,
      pattern: ".generated/content/docs/**/*.{md,mdx}",
      generateId,
    }),
    schema: docsSchema(),
  }),
};
