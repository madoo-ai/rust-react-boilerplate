# 0002 — Axum API and React SPA

- Status: accepted
- Date: 2026-11-12

## Context

The source is a Next.js app with Server Components and Server Actions. The port needs a Rust API and a React client. Server-rendered React would require a Rust SSR layer that has no strong ecosystem equivalent. The product is an authenticated app, not an SEO-first site.

## Decision

Use an Axum JSON API and a React single-page application built with Vite.

- Vite proxies `/api` to Axum in development.
- The release binary serves `web/dist` with an SPA fallback.
- The client uses React Router and TanStack Query.
- Mutations call the JSON API. There are no server actions.
- The API remains usable by a second client.

## Alternatives

- Server-rendered React in Rust: immature, large surface, no clear win for an authenticated app.
- Keeping Next.js and adding a Rust sidecar: two runtimes and two data paths for one product.
- Leptos or Dioxus: a full Rust UI, but the team wants React and the shadcn ecosystem.

## Consequences

- The API is a real contract with tests.
- The first paint of a locale change happens on the client.
- The deployment has two build steps: web assets and one binary.
- The frontend can be hosted separately when `ALLOWED_ORIGINS` includes its origin.
