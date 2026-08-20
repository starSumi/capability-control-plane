# Security input context

These local documents were read as design proposals and threat-model prompts.
They are not vendored, executed, or treated as standards:

| Input | SHA-256 | Use | Boundary |
| --- | --- | --- | --- |
| `k8s-philosophy.md` | `2B94AE67E8F56FB83A8AAD9264E668EE8D95827E35BF7F693567EF7321761D20` | level-triggered reconciliation and spec/status review | proposal |
| `OWASP GenAI LLM Top 10 (2026).md` | `F50C93C95AD728DAF9D8CC0A0BAEFC532CC48E4FFE80C981EA6FC75918B983BB` | agent trust-boundary prompts | local synthesis |
| `CWE SANS 顶级软件缺陷病理与全域防御拓扑.md` | `C25E366C01A03F303D6C1A6A2F0089F89E6840B5CC35494D29B971F31ACA765E` | defect-class review prompts | local synthesis |

Hashes were observed on 2026-08-20. Refreshing a source requires a new review
round; it must not silently change an accepted decision.

