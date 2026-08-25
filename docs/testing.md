# Testing and evaluation

Tests are layered so a green unit test cannot be mistaken for production proof.

## Required layers

1. **Protocol**: strict unknown-field rejection, enum compatibility, malformed
   versions, fixed-width integer negative/fraction/overflow cases, duplicate
   identities, JSON Schema generation, TypeScript mapping, and exact
   generated-file drift checks.
2. **Engine**: authority and permission denial, hard bounds, Unicode/CJK
   normalization, deterministic ordering, explicit identity, ambiguity,
   no-match, top-1 ambiguity using an internal runner-up, quantized score
   stability, stale generation/resource version, policy binding, full CAS,
   orphan retention, and pure planning.
3. **Boundary**: root-relative paths, parent traversal, symlink/reparse
   rejection, stable-handle reads, document byte limits, policy validation
   before dependent I/O, malformed JSON, and structured exit codes.
4. **Property/fuzz**: bounded random text and catalog permutations must never
   panic, execute, widen permissions, or produce nondeterministic output.
5. **Cross-platform**: Linux, Windows, and macOS CI are configured to run the
   locked workspace tests. Any cross-platform determinism claim additionally
   requires route-golden fan-in and byte comparison. The frozen
   `fixtures/route-golden.v1alpha1.json` corpus is rendered by the pure engine
   example and compared byte-for-byte by the `route-golden-fan-in` CI job.
   Windows path behavior is a first-class contract, not an afterthought.
6. **Adversarial**: injection-looking descriptions, shell metacharacters,
   path encodings, oversized Unicode, duplicate resource versions, and hostile
   provider labels remain data and cannot become commands.

## Shadow replay contract

`capctl shadow` replays `fixtures/shadow-heldout.v1alpha1.json` against one
catalog, policy, and selector configuration. The fixture revision is a BLAKE3
digest of its canonical metadata, provenance, selector, and query-bearing cases;
changing any case invalidates the replay. The output records catalog, policy,
selector, and fixture identities plus aggregate decision/Hit@K/false-positive
counts. It deliberately omits raw query text and emits no provider reads,
client mutation, or desired-state changes.

The checked-in seven-case fixture is a seed-only manually reviewed corpus. Its
`heldOutReview` label is provenance, not an independent production ground truth.
Promotion requires a separately owned, independently reviewed corpus, native or
provider actual-use evidence, version/digest slicing, retention/redaction rules,
and representative query volume. Do not convert this seed result into an SLO.

The protocol validator and generated Schema share the Rust source but do not
have identical expressive power: byte-oriented and cross-field checks must be
covered by Rust negative fixtures, while generated schemas are checked for the
structural subset they advertise. A release candidate also needs a held-out
route corpus, catalog-digest slicing, and allocation/RSS measurements at the
10,000-descriptor and high-Unicode ceilings.

## Acceptance language

“Pass” means the named command ran on the named revision and produced the
reported result. Static inspection, a local smoke test, or a model benchmark is
not release evidence. A production promotion additionally needs cold-start and
warm-path measurements on representative hosts, a held-out query set, supply
chain checks, artifact provenance, and a rollback rehearsal.
