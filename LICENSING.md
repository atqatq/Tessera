# Licensing and contribution policy

The licence is plain [Apache-2.0](LICENSE). This file states, in plain
language, the parts of the legal posture a reader would otherwise have
to parse out of the licence text. It is informative — the licence,
the DCO, and the actual agreements govern.

## The patent position (why plain Apache-2.0 matters here)

Infrastructure lives downstream of other infrastructure, so patent
exposure decides adoption. Apache-2.0 handles it with two clauses:

- **§3, the patent grant.** Every contributor automatically grants
  every user a licence to their patents that cover their contribution.
  You do not negotiate it, accept it separately, or pay for it —
  contributing to Tessera grants it to everyone who uses Tessera,
  forever, worldwide, at no charge.
- **§3, the termination.** If you sue anyone claiming Tessera
  infringes your patents, your patent licence from every Tessera
  contributor terminates as of the day the suit is filed. You keep
  the copyright licence; you lose the patent shield. Deterrence sits
  in the licence instead of in a CLA.

That is why the project is Apache-2.0 **alone** and will not dual-
licence under MIT (which carries no patent grant and would let
downstream elect out of the protection), and why the old naming
appendix was removed (a bespoke condition jeopardised the whole
classification). The name is a trademark matter, not a licence
condition — see [TRADEMARK.md](TRADEMARK.md).

## Contributions: DCO, not a CLA

Contributions are licensed under Apache-2.0 via the **Developer
Certificate of Origin** (DCO 1.1): every commit carries a
`Signed-off-by` line (`git commit -s`) certifying you have the right
to submit it. CI enforces the sign-off.

Why DCO instead of a CLA:

- A CLA transfers or licenses extra rights to one entity; the DCO
  certifies provenance and leaves rights with their authors.
- Individual contributions stay individual; companies stay comfortable
  because their employees' sign-off is the standard certification
  every git workflow already supports.
- No gate, no paperwork lag, no signature infrastructure to run.

## Third-party code policy

- Dependencies must be permissively licensed (Apache-2.0, MIT,
  BSD-3-Clause, ISC, Unicode-3.0, Zlib at present). `cargo deny`
  enforces the allowlist in CI; the list is mirrored in
  [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).
- New dependencies need a justification in the PR, a maintenance
  check, and a note on the cost of removing them (Part A5).
- Vendored assets (fonts, artwork) carry their licences in-file and in
  THIRD_PARTY_LICENSES.md.
- Inbound contributions are licensed under the project licence via the
  DCO — see above.

## Corporate CLA requests

Some companies require a signed CLA before their staff may contribute.
Requests will be evaluated honestly: the project will accept a *corporate
DCO acknowledgement* (the company confirms its employees may sign off)
but will not adopt a CLA that transfers rights or adds grant conditions,
because that would defeat the patent and governance posture above. If a
company needs more than a DCO acknowledgement, their staff may
contribute through issues and vectors instead, and that will be stated
as a real cost, not hidden.

## Export control note

The kernel ships (and will ship) hashing and signature verification.
Several jurisdictions treat cryptographic implementations in published
open-source projects as subject to notification or classification
rules, though most also carve out published open-source code. **This
warrants a lawyer's opinion before the first tagged release**, and
drafting that opinion is the maintainer's task with counsel — not
something generated for the repository. This note exists so the
question is asked before the first release rather than after.
