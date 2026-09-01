# Design tokens

The single source of truth is
[`brand/tokens.json`](https://github.com/atqatq/Tessera/blob/main/brand/tokens.json);
this page restates the parts people ask about. A second palette
anywhere — docs, site, frontend — is a bug, not a variant.

## Colour

- Monotone **zinc** ramp (`#fafafa` → `#09090b`).
- Exactly **one accent: `#6E96E8`** — reserved for the thing being
  placed: the active state, the recommendation, the focal element.
- Functional state colours (`risk` `#C84B41`, `success` `#4F8A5B`)
  exist for semantics, never decoration.
- Light and dark themes swap ink/background only; WCAG AA contrast is
  checked per theme, and meaning is never carried by colour alone.

## Type

- Text: **Inter** (variable; self-hosted, SIL OFL 1.1).
- Mono: **JetBrains Mono** (self-hosted, SIL OFL 1.1) — code,
  identifiers, hashes.
- Scale: 12 / 14 / 16 / 20 / 24 / 32 / 48. Sizes between steps are not
  allowed.

## Space and radius

4px base grid; legal gaps are 4, 8, 12, 16, 24, 32, 48, 64. Radius:
10 for cards, 6 for controls, 4 for icons.

## Files

| Asset | Path |
|---|---|
| Mark (light / dark / accent) | `brand/mark-{light,dark,accent}.svg` |
| Favicon | `brand/favicon.svg` |
| Tokens | `brand/tokens.json` |
| Usage rules | `BRAND.md` |
| Fonts + licences | `brand/fonts/` |
