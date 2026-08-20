# Threat model

## Assets

- desired catalog and policy integrity;
- authority and permission boundaries;
- deterministic route and plan integrity;
- operator privacy and redacted diagnostics;
- dependency, CI, and artifact provenance.

## Trust boundaries

```text
untrusted query/content -> protocol -> pure engine -> structured decision
client input -> rooted reader -> strict JSON resources
policy writer -> admission -> plan -> future provider/actuator
dependency source -> Cargo.lock/deny -> build -> artifact
```

The engine does not cross into a provider, shell, process, network, client, or
filesystem. A future adapter adds a new boundary and must extend this model
before implementation.

## Threats and controls

| Threat | Control | Residual risk |
| --- | --- | --- |
| prompt injection in descriptions | content affects rank only; no tool execution | provider assembler must re-review |
| authority confusion/path traversal | opaque locator; rooted relative reader; indirect paths denied; stable file-handle read with post-open name check | a privileged actuator needs a handle-relative rooted opener; current reader still assumes no hostile root-directory renames |
| policy widening | compiled hard ceilings and typed permission set | policy file owner is external to V1 |
| stale overwrite/replay | desired/observed resource version, policy identity/digest, and per-item generation/digest CAS preconditions | actuator must re-observe and enforce them atomically |
| resource exhaustion | document, catalog, field, collection, query/descriptor term, text-byte, and top-K ceilings | allocation profiling still needs workload baselines |
| supply-chain compromise | lockfile, source/license deny, pinned CI actions | advisory freshness depends on CI access |
| diagnostic leakage | route output contains no query text, matched terms, or stable query digest; errors exclude content | CLI arguments and host logs remain operator responsibility |
| model overclaim | no semantic confidence magic number; explicit ambiguous/no-match; fixed-target determinism wording | retrieval quality and cross-platform golden identity need held-out data |

## Abuse cases

Test fixtures must cover unknown JSON fields, fake permissions, duplicate names,
malicious Unicode, shell metacharacters, absolute and parent paths, symlink and
reparse traversal, oversized documents, stale generations, duplicate observed
records, query canaries, fixed-width integer overflow, and catalog permutation.
The expected result is a structured denial or the same deterministic plan,
never a panic or side effect.
