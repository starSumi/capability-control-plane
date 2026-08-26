# Security Policy

## Supported versions

The project is pre-1.0. Security fixes target the latest commit on `main`; no
released compatibility window exists yet.

## Reporting a vulnerability

Do not open a public issue containing exploit details, secrets, personal data,
or a working proof of concept. Use [GitHub private vulnerability reporting] and
include:

- affected revision and platform;
- trust boundary crossed and required preconditions;
- minimal reproduction with sensitive values removed;
- impact, evidence, and a proposed safe disclosure window.

The project does not claim a response SLA before a public maintainer roster and
measured response process exist. This avoids publishing a fictitious promise.

[GitHub private vulnerability reporting]: https://github.com/starSumi/capability-control-plane/security/advisories/new

## Security boundary

V1 is a read-only discovery and planning kernel. It does not execute a selected
capability, resolve provider content, mutate a client, or run a network service.
Selection is not authorization. See [the threat model](docs/security/threat-model.md)
and [hardening portfolio](docs/security/hardening.md).
