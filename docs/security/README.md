# Security architecture

The security posture is fail-closed and evidence-bounded. The local Vault
documents used to seed this design are synthesis inputs, not normative sources;
their hashes and accepted/rejected claims are recorded in
[`context.md`](context.md).

Primary external references:

- [OWASP Top 10 for Agentic Applications 2026](https://genai.owasp.org/resource/owasp-top-10-for-agentic-applications-for-2026/)
- [MITRE CWE Top 25 2025](https://cwe.mitre.org/top25/archive/2025/2025_cwe_top25.html)
- [SLSA v1.2](https://slsa.dev/spec/v1.2/)
- [GitHub Actions security hardening](https://docs.github.com/en/code-security/tutorials/secure-your-organization/protect-against-threats)

This project adopts principles from those sources where they map to V1 trust
boundaries. It does not claim compliance or immunity.

