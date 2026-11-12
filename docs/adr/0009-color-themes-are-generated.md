# 0009 — Color themes are generated

- Status: accepted
- Date: 2026-11-12

## Context

The app needs the OpenCode desktop theme catalogue without hand-maintaining 37 palettes and their light and dark variants.

## Decision

Keep the mode and the palette separate.

- `theme` cookie: `light`, `dark` or `system`.
- `color-theme` cookie: the default Stripe palette or an OpenCode theme id.
- `web/src/lib/theme/opencode-themes.json` is the palette source with its commit metadata.
- `web/scripts/generate-themes.ts` generates:
  - `web/src/styles/themes.css`
  - `web/src/lib/theme/theme-options.ts`
- The default palette stays in `:root` and `.dark`.
- The test suite checks the option ids and a generated palette.

## Alternatives

- One flat list of 74 variants: doubles the picker and loses the independent mode switch.
- Runtime CSS-in-JS: more client work and a flash risk.
- Hand-written themes: unmaintainable.

## Consequences

- The picker has one option per theme.
- The generated CSS is about 3,500 lines.
- Updating the catalogue is a data change and a generator run.
