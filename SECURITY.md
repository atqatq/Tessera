# Security Policy

## Supported

| Branch | Status |
|---|---|
| main / latest release | supported, security fixes |
| older releases | best effort, 90 days |

## Reporting

Email **security@tessera-scm.dev** (private, encrypted ok). Please include
affected component (`hub.*` service or spoke code), a repro or conformance
vector, and impact assessment. Do not open public issues for
vulnerabilities.

- Acknowledgement within 72 hours; triage within 7 days.
- Coordinated disclosure, 90-day window, credit unless you prefer otherwise.
- Please respect data: no exfiltration of tenant data in testing.

## Guarantees the codebase is held to

- ORIGIN is above root but **never** bypasses L0 (spoke state) or the ledger;
  every ORIGIN action is intent-logged before effect (hardware key +
  threshold signature).
- Agents hold no credentials; act-tier is allowlist + ORIGIN approval.
- Cross-party writes do not exist as an operation.
- Ledger and master log are append-only, hash-chained per tenant.
