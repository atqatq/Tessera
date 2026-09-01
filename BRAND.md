# Brand

Tessera means **a single tile in a mosaic**. Every participant — a
tenant, a module, a person — places one tile into a larger picture they
do not wholly see. The mark is that idea; the palette refuses to
improve on it.

This file governs usage. `brand/tokens.json` is the machine-readable
source of truth — the docs site and the future frontend consume it, and
a second palette anywhere is a bug, not a variant.

## The mark

`brand/mark-light.svg`, `brand/mark-dark.svg`, `brand/mark-accent.svg`,
`brand/favicon.svg`.

A 2×2 mosaic: three tiles set, one drawn as an outline — the tile being
placed. Pure geometry, one colour, legible at 16px. The favicon *is*
the mark; no simplified variant exists because none is needed.

- Render on a plain background with clear space on all sides.
- Do not rotate, skew, outline, gradient, shadow, or re-tile it.
- Do not place it inside a shape that crowds the clear space.
- Do not replace the outlined tile with a filled one — the missing
  tile is the idea.

## Wordmark

Set "Tessera" in the text face (Inter), semibold, sentence case — never
all-caps, never stretched. The mark plus the wordmark is the lockup;
the mark alone is the favicon and the avatar.

## Palette — monotone zinc, exactly one accent

The zinc ramp is in `tokens.json`. Rules:

1. **One accent, `#6E96E8`.** It marks the thing being placed: the
   active state, the recommendation, the focal element. If everything
   is accented, nothing is.
2. **State colours are functional, not decorative.** Risk red and
   success green appear only where state must be communicated.
3. **No other colours.** Gradients of the accent, tinted greys, and
   "just one more" hues are all the same mistake.
4. Dark and light themes swap ink and background only — accents and
   semantics keep their meaning across themes (contrast is adjusted to
   WCAG AA, checked per theme).

## Type

- **Text: Inter** (self-hosted, `brand/fonts/InterVariable.woff2`,
  SIL OFL 1.1). Weights 400–700.
- **Mono: JetBrains Mono** (self-hosted, SIL OFL 1.1) for code,
  identifiers, hashes, and anything from the ledger.
- The type scale (12→48) and line heights are in `tokens.json`; sizes
  between steps are not allowed. Half of readability is restraint.

## Spacing and radius

4px base grid; the named steps in `tokens.json` are the only legal
gaps. Radius: 10 for cards, 6 for controls, 4 for icons. Mixing radii
within one surface is a bug.

## Accessibility

- WCAG AA contrast on every text/background pair, in both themes.
- Meaning is never carried by colour alone (state gets an icon or a
  label as well).
- The mark always carries alt text that says what it is, not what it
  looks like.

## Misuse — the short list

No attribution-style credit lines are required anywhere (the licence
has no such condition); using the mark to *state truthfully* that
something runs on Tessera is nominative use and always fine. What is
not fine is in TRADEMARK.md: presenting a fork as Tessera, implying
endorsement, trading on the name.
