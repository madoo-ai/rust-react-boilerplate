# 0005 — Cookie locale and theme

- Status: accepted
- Date: 2026-11-12

## Context

The source stores locale and appearance in cookies and reads them on the server before rendering. The SPA has no server-rendered HTML for app pages.

## Decision

Keep the cookies, and apply the theme before paint with an inline script.

- `locale` cookie: `id` or `en`. The SPA reads it at boot.
- `theme` cookie: `light`, `dark` or `system`.
- `color-theme` cookie: the default palette or an OpenCode theme id.
- `web/index.html` sets the dark class and `data-color-theme` before the first paint.
- The React providers update the class and the cookies on change.
- There is no `/[locale]` segment.

## Alternatives

- `localStorage`: the inline script could read it, but cookies keep one storage mechanism for theme and locale.
- Server-side locale rendering: correct, but it needs an SSR layer the port does not have.
- URL-prefixed locales: more SEO control, unnecessary for an authenticated app.

## Consequences

- The theme does not flash.
- The first locale render uses the default text until React reads the cookie. Keep the default layout stable.
- Every page reads the cookies on the client. That is acceptable for this app.
