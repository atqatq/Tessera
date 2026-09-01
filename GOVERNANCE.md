# Governance

Tessera is developed in the open under the Apache-2.0 license with the
[Tessera naming requirement](LICENSE#appendix--tessera-naming-requirement).
This document describes who decides what, how authority is earned, and how
conflicts are resolved. It is a living document: amend it by pull request.

## Long-term home: the Eclipse Foundation

The project's long-term home is the **Eclipse Foundation**. Moving an
infrastructure project to a vendor-neutral foundation is how Tessera stays
governed by its users rather than by any single company. Until the
foundation transition completes, this repository is the canonical home and
the governance below applies as written.

Concretely, the path to Eclipse looks like this:

1. **Now (pre-founding).** The project runs on lazy consensus with the
   roles defined below. Contributor License Agreements are *not* used;
   the [DCO](#developer-certificate-of-origin) is.
2. **Proposal.** Once the committer base is broad enough to satisfy the
   Eclipse Development Process, the project is proposed to the Eclipse
   Foundation (Architecture Council sponsorship, project creation review).
3. **Adoption.** On acceptance, this repository migrates, the Eclipse
   Development Process (EDP) replaces the process below, and the naming
   requirement is reconciled with the Eclipse Trademark Policy. The
   Apache-2.0 license carries over unchanged.

Nothing in the transition may weaken the license, the naming requirement,
or the deny-wins security posture.

## Roles

| Role | Who | Powers |
|---|---|---|
| **User** | anyone running or evaluating Tessera | open issues, vote on roadmap candidates |
| **Contributor** | anyone with a merged PR | propose changes, review PRs |
| **Committer** | contributor with sustained, high-quality merges | merge PRs, shape releases |
| **Project Lead** | committers elected by committers | repo administration, final tie-breaks, security response |

- **Becoming a committer.** Sustained contribution over months — code,
  conformance vectors, reviews, or documentation. Existing committers
  nominate; committers confirm by lazy consensus.
- **Losing committership.** Twelve months of inactivity triggers a
  courtesy review; committers are never removed for dissent.

## Decision making

- **Lazy consensus** is the default: a proposal stands unless a committer
  objects with reasons. Silence is consent; objections are resolved by
  discussion, then by committer vote, then by the Project Lead as the
  last tie-break.
- **Binding decisions** that require a supermajority of committers:
  license or naming-requirement changes, new hub services in the
  `requires` allowlist, new spokes in the register, and any change to
  this document.
- **No decision by ambush:** material decisions are announced in a
  dedicated issue and stay open at least five days before a vote closes.

## Developer Certificate of Origin

Every commit must be signed off — `git commit -s` — certifying that the
contributor has the right to submit the code under the project license
([DCO 1.1](https://developercertificate.org/)). The CI gate rejects
unsigned commits. This is the project's alternative to a CLA: lighter for
contributors, still an auditable paper trail.

Signed commits (GPG or SSH) are **enforced on `main`** so the release
history is verifiable end to end. Day-to-day development happens on
feature branches where signing-off is required but cryptographic commit
signatures are not.

## Change workflow

1. Open an issue or pick one. Large changes need a proposal issue first
   (see [CONTRIBUTING.md](CONTRIBUTING.md)).
2. Branch from `main`, one logical change per pull request.
3. Every gate green: `make check` (fmt, clippy `-D warnings`, rust and
   python suites, conformance vectors, manifest checks, REUSE).
4. Review by at least one committer who did not author the change.
   Behavior changes additionally require conformance vectors — the
   vector-first rule is not negotiable.
5. Squash-merge or rebase; commits keep their DCO sign-off.

## Security and the trust boundaries

The permission model (five layers, deny wins), the ledger, and the grid
contracts are security-relevant by construction. Changes touching them:

- require a committer review focused on the failure mode, not the happy
  path;
- must not introduce a default-allow path — such a PR is rejected on
  principle, however correct the rest of it is;
- cryptographic code is **out of scope by policy**: hashing and signing
  ride on widely audited libraries, and key custody stays with KMS/HSM
  integrations. The project does not invent ciphers, signature schemes,
  or key exchange.

Report vulnerabilities through the private channel in
[SECURITY.md](SECURITY.md) — never in public issues.

## Code of conduct

The [Contributor Covenant](CODE_OF_CONDUCT.md) applies to every space
where the project is discussed, including issue threads and reviews.
Enforcement starts with the Project Lead and escalates to committers as
a group.
