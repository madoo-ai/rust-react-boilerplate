# 0010 — The root path is a public landing page

- Status: accepted
- Date: 2026-11-12

## Context

A boilerplate that requires a session before the first screen is hard to evaluate and gives the theme system no public place to show itself.

## Decision

Split the public and authenticated entry points.

- `/` is a public landing page.
- `/dashboard` is the authenticated home.
- Login, registration, 2FA and redirects point to `/dashboard`.
- The landing page uses the real theme data, a live palette rail, a dashboard preview and the seeded demo accounts.
- Marketing headings use Instrument Serif. Product screens use Inter.
- Motion is CSS-only and gated by `prefers-reduced-motion`.

## Consequences

- Guests can evaluate the design and features.
- Every authenticated link uses `/dashboard`.
- The landing page imports the same theme data as Settings, so the palettes cannot drift.
