# Releases

## Cadence

Once v0.1 ships, releases are **time-based, not feature-based**:

- **Minor** every six weeks (`0.x`), on the advertised date even if
  the headline feature slipped — a train you can stand on, not a
  surprise party.
- **Patch** as needed for regressions and security fixes.
- Versioning is SemVer; the public-API diff check runs per release
  once crates stabilise.

Release notes are generated from Conventional Commits and never
invent content: if a change did not say why, the notes cannot say
what. The CHANGELOG is the human-readable record; this file is the
promise.

## Deprecation policy

- A deprecated API or behaviour is announced in the CHANGELOG and the
  docs, kept working for **at least two minor releases**, and removed
  only after both windows pass.
- Deprecations warn in code (where a lint can carry them) and in the
  release notes (where humans read).
- Breaking changes to the frozen module manifest schema require a new
  schema version and a migration path — never a silent cut-over.

## Support windows

- Latest minor: all fixes.
- Previous minor: security fixes only, for one minor cycle.

## What a release must contain

- The full suite green (`make check` parity with CI).
- The CHANGELOG updated, with the date.
- The release workflow output: CycloneDX SBOM, provenance attestation,
  keyless signatures (see `.github/workflows/release.yml`).
- The SLSA claim in the README updated to match what actually shipped.
