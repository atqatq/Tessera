# When not to use Tessera

This page should be more persuasive than the "why" page, because it
costs the project something to write.

## Do not use Tessera today if you need running supply chain software

What is built and tested is the kernel — permission engine, ledger,
identifiers — plus the `inv` safety-stock core, consumed through
libraries and conformance vectors. There is no deployable application,
no UI, no ingest pipeline yet. Check the
[status table](https://github.com/atqatq/Tessera#what-exists-today)
and the [ROADMAP](https://github.com/atqatq/Tessera/blob/main/ROADMAP.md);
if you need MES or WMS or EDI in production this year, buy something
that exists.

## Do not use it if you want a turnkey AI product

Agents in Tessera are tiered, allowlisted, and approval-gated; they
propose, humans commit. If you want autonomous software that acts
without review, Tessera's design will frustrate you deliberately.

## Do not use it if your tenancy model is single-company

Much of the kernel exists to make *untrusted neighbours* safe: per-
tenant chains, peer-read grants, the party boundary, the grid specs.
A single-tenant deployment gets little from that complexity — a plain
database with an audit table would serve you better, and that is not
an insult, it is proportionality.

## Do not use it if you cannot accept default-deny friction

Every new column, module, or integration starts closed and needs an
explicit rule ([why](why-deny-wins.md)). Teams that want "just let
everyone read everything during the pilot" will fight the engine, and
the engine will win, and then you will be unhappy together.

## Do not use it if you need to modify supply chain method per tenant

The line is drawn deliberately: **structure is configurable, method is
not**. A tenant defines entities, flows, and vocabulary; a tenant does
not redefine how safety stock is computed. If your competitive edge
depends on a secret, home-grown planning algorithm, Tessera's
opinionated core will be a straightjacket, not a foundation.

## What would change the answer

If any of the "not yet" items matches your need, watch the roadmap —
or better, run the [test suite](../tutorials/first-suite.md), read the
[ADRs](../adr/index.md), and tell the project which constraint is wrong.
That is what the RFC process is for.
