const documents = [
  {
    source: "README.md",
    target: "README.md",
    slug: "index",
    title: "Capability Control Plane",
    label: "Overview",
    description: "A fail-closed authority kernel for capability catalogs, policies, routes, and reconcile plans.",
  },
  {
    source: "docs/architecture.md",
    target: "docs/architecture.md",
    slug: "architecture",
    title: "Architecture",
    label: "Architecture",
    description: "Physical model, ownership boundaries, and pure-kernel topology.",
  },
  {
    source: "docs/lifecycle.md",
    target: "docs/lifecycle.md",
    slug: "lifecycle",
    title: "Lifecycle",
    label: "Lifecycle",
    description: "Desired state, observation, planning, verification, and recovery.",
  },
  {
    source: "SECURITY.md",
    target: "SECURITY.md",
    slug: "security",
    title: "Security Policy",
    label: "Security",
    description: "Security policy and fail-closed reporting boundary.",
  },
  {
    source: "docs/security/threat-model.md",
    target: "docs/security/threat-model.md",
    slug: "security/threat-model",
    title: "Threat model",
    label: "Threat model",
    description: "Assets, trust boundaries, abuse cases, and security invariants.",
  },
  {
    source: "docs/security/hardening.md",
    target: "docs/security/hardening.md",
    slug: "security/hardening",
    title: "Hardening portfolio",
    label: "Hardening portfolio",
    description: "Evidence-backed structural security improvements.",
  },
  {
    source: "docs/testing.md",
    target: "docs/testing.md",
    slug: "testing",
    title: "Testing and evaluation",
    label: "Testing",
    description: "Verification layers and acceptance language.",
  },
  {
    source: "docs/benchmarks.md",
    target: "docs/benchmarks.md",
    slug: "benchmarks",
    title: "Benchmarks",
    label: "Benchmarks",
    description: "Benchmark workloads, evidence, and interpretation rules.",
  },
  {
    source: "docs/roadmap.md",
    target: "docs/roadmap.md",
    slug: "roadmap",
    title: "Roadmap",
    label: "Roadmap",
    description: "Evidence-gated product evolution.",
  },
];

export const humanPages = Object.freeze(documents.map(Object.freeze));
export const sitePages = humanPages;
export const projectionDocuments = Object.freeze(
  documents.map(({ source, target }) => Object.freeze({ source, target })),
);
