# 0006 — One UI primitive layer

- Status: accepted
- Date: 2026-11-12

## Context

The source uses shadcn/ui on Radix. The port needs the same components, form conventions and dark-mode behavior.

## Decision

Use shadcn/ui as the only primitive layer.

- The primitives live in `web/src/components/ui`.
- App-specific composition lives in `components/common`, `app-shell`, `dashboard`, `users`, `roles`, `settings`, `auth` and `marketing`.
- Use `Field` primitives for forms and `AlertDialog` for destructive confirmation.
- Use semantic tokens, not raw palette classes.
- `DESIGN.md` is the visual contract.
- Do not add a second component library.

## Consequences

- The project owns the component source.
- Upgrades use the shadcn CLI diff workflow.
- The UI has one set of tokens, radii and interaction states.
