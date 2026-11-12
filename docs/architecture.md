# Architecture

## Request lifecycle

1. Vite serves the React app in development. It proxies `/api` to Axum. In production, Axum serves `web/dist`.
2. `web/src/App.tsx` guards authenticated routes. The guard reads the session from `/api/auth/session`.
3. Axum resolves the session cookie, loads the user, role and permissions from PostgreSQL and attaches `CurrentSession`.
4. The handler calls a use case. The use case re-checks the permission and the fine-grained policy.
5. The use case writes through a driven port. The SQLx adapter owns the transaction and converts rows back to entities.
6. The handler maps the result to JSON. The React client refreshes the affected TanStack Query keys.

The cookie is never proof of a valid session. Every request reloads the session from the database.

## Layers

```text
handler/httpapi -> app/usecase -> app/port <- adapter/*
                       |
                    entity
```

| Layer     | Owns                                                                 | Must not import                     |
| --------- | -------------------------------------------------------------------- | ----------------------------------- |
| `entity`  | Domain types, validation, policies                                     | `axum`, `sqlx`, `lettre`            |
| `port`    | Driven-port traits and port errors                                    | Adapters, HTTP types                |
| `usecase` | Business rules and orchestration                                      | Adapters, `axum`, `sqlx`            |
| `adapter` | SQLx queries, SMTP, Argon2, HMAC, TOTP, clock, IDs, rate limiting    | HTTP handlers, use cases            |
| `handler` | Routing, extractors, cookies, status codes, DTOs                      | SQL, business rules                 |
| `xmodule` | Configuration and wiring                                              | Nothing; it is the composition root |

## Project structure

```text
api/
  migrations/                  SQLx migrations
  src/
    entity/                    domain types and policies
    app/port/                  driven-port traits
    app/usecase/               auth, users, roles, settings, dashboard
    adapter/postgres/          SQLx repositories and seed
    adapter/crypto/            Argon2, HMAC, TOTP, clock, IDs, limiter
    adapter/smtp/              lettre mailer
    handler/httpapi/           axum router, handlers, middleware, DTOs
    xmodule/                   config and dependency wiring
    lib.rs                     router and module tree
    main.rs                    thin CLI: serve, migrate, seed
web/
  src/
    pages/                     marketing, auth, app and settings pages
    layouts/                   marketing, auth and app shell layouts
    components/                app shell, auth, users, roles, settings, UI
    lib/                       API client, auth, i18n, theme, format, errors
    messages/                  en.json and id.json
    styles/                    globals.css, themes.css, shadcn.css
    config/                    app name and navigation
    types.ts                   shared client types
  tests/                       unit and Playwright tests
docs/                          product docs and ADRs
.agents/skills/rust-skills/    Rust coding rules for agents
```

## Server and client boundaries

- The API is the only writer. The browser never touches PostgreSQL.
- The session cookie is HTTP-only. JavaScript cannot read it.
- DTOs are separate from entities. Do not send database rows to the browser.
- TanStack Query owns server state. Do not copy server data into long-lived client stores.
- Forms use React Hook Form and Zod. The API validates again. Client validation is a convenience, not a security control.

## Authorization

Authentication answers "who is this?". The application answers "may they do that?".

- `CurrentSession` is the HTTP extractor. It loads the session, user, role and permissions.
- `Actor::require` is the coarse gate inside every use case.
- `entity/authorization.rs` holds the fine-grained policies.
- The sidebar hides links that the user cannot use. Hiding is never the only protection.

## Validation and errors

- Use cases validate input and return `DomainError` variants with i18n keys.
- The handler maps `AppError` to a status code and the shared JSON envelope.
- Validation failures return HTTP 422 with a `fields` map.
- Unexpected failures log the source chain and return `errors.unexpected`.
- The client translates message keys. It never displays a raw SQL or internal error.

## Data

- PostgreSQL is the only database.
- IDs are ULIDs stored as text.
- Timestamps are `timestamptz`.
- The migration set is embedded with `sqlx::migrate!`.
- The seed is idempotent. It upserts permissions, roles and the approval setting, then creates demo users that do not exist yet.
- Session tokens are stored as SHA-256 hashes. Backup codes are stored as SHA-256 hashes in JSONB.

## Frontend routing

React Router owns the route tree. The route groups mirror the product:

- public landing page
- auth pages
- authenticated app shell
- settings area
- error pages

TanStack Query keys are stable and scoped:

- `session`
- `dashboard`
- `users` with the current filters
- `user-meta`
- `user` with the id
- `roles`

Mutations invalidate the keys they change.

## Quality gates

`make lint` runs:

1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. ESLint
4. TypeScript
5. i18n key parity

`make test` runs Cargo tests and Vitest. `make test-e2e` runs Playwright against the test database.
