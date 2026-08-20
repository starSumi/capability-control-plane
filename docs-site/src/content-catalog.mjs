const pages = [
  { label: "Overview", slug: "index" },
  { label: "Architecture", slug: "architecture" },
  { label: "Security", slug: "security" },
  { label: "Testing", slug: "testing" },
  { label: "Roadmap", slug: "roadmap" },
];

const machineDocuments = [
  { source: "README.md", target: "README.md" },
  { source: "SECURITY.md", target: "SECURITY.md" },
  { source: "docs/architecture.md", target: "docs/architecture.md" },
  { source: "docs/benchmarks.md", target: "docs/benchmarks.md" },
  { source: "docs/lifecycle.md", target: "docs/lifecycle.md" },
  { source: "docs/roadmap.md", target: "docs/roadmap.md" },
  { source: "docs/testing.md", target: "docs/testing.md" },
  {
    source: "docs/security/threat-model.md",
    target: "docs/security/threat-model.md",
  },
  {
    source: "docs/security/hardening.md",
    target: "docs/security/hardening.md",
  },
];

export const sitePages = Object.freeze(pages.map(Object.freeze));
export const projectionDocuments = Object.freeze(
  machineDocuments.map(Object.freeze),
);
