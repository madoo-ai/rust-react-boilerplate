# Rust React Boilerplate

Opinionated Rust + React boilerplate with an Axum API, SQLx, PostgreSQL, RBAC, i18n and a shadcn/ui shell.

This is the Rust sibling of the Next.js and Laravel React boilerplates. It keeps the core: auth, roles and permissions, user management, settings, locale, theme and the app shell. Employee documents, attendance and leave requests are deliberately left out so the next project can add its own domain.

## What's inside

- **Auth**: email/password, email verification, password reset, TOTP two-factor authentication, backup codes, trusted devices, session management, rate limiting and a confirm-password gate.
- **Registration approval**: optional approval mode. New accounts stay `pending` until an HR user verifies them.
- **RBAC**: three seeded roles (`superadmin`, `hr`, `user`), a permission registry, user and role management, a protected superadmin and privilege-escalation checks.
- **Users and roles**: searchable, filterable and paginated users; create, edit, verify and delete; role permission matrix; protected system roles.
- **Settings**: profile, password, appearance, security (2FA) and account deletion.
- **i18n**: Indonesian default and English. The `locale` cookie is the source of truth.
- **Appearance**: light/dark/system, the Stripe-inspired default palette and 37 OpenCode color themes.
- **Landing page**: a public entry point with a live theme lab, a dashboard preview, the feature tour and the demo accounts.
- **Frontend**: React 19, Vite, React Router, TanStack Query, Tailwind CSS 4, shadcn/ui, Altics icons, Recharts and Sonner.
- **Data**: PostgreSQL 18, SQLx migrations, an idempotent seed and ULIDs.
- **Quality**: rustfmt, Clippy, ESLint, Prettier, TypeScript strict, Vitest, Playwright, i18n key parity and GitHub Actions.
- **Ops**: multi-stage Dockerfile, Compose stack with Postgres and Mailpit, Makefile and a rename script.
- **Agent docs**: `AGENTS.md`, product docs under `docs/`, ADRs and the Rust skill under `.agents/skills/`.

## Stack

| Layer         | Choice                                                     |
| ------------- | ---------------------------------------------------------- |
| API           | Rust 1.96, Axum 0.8, Tokio                                 |
| Data          | SQLx 0.9, PostgreSQL 18                                    |
| Auth          | Argon2id, opaque database sessions, TOTP, SMTP             |
| Authorization | App-owned RBAC tables and policies                         |
| UI            | React 19, Vite 8, React Router 7, Tailwind CSS 4, shadcn/ui |
| Data fetching | TanStack Query 5, Zod 4, React Hook Form 7                 |
| i18n          | Cookie locale, typed message catalogues                    |
| Tests         | Cargo test, Vitest, Playwright                             |
| Runtime       | Docker, Compose                                            |

## Architecture

The Rust side uses ports and adapters:

```text
handler/httpapi -> app/usecase -> app/port <- adapter/*
                       |
                    entity
```

- `entity` holds pure domain types and policies.
- `app/port` holds the driven-port traits.
- `app/usecase` holds the business logic. It depends on entities and ports only.
- `adapter` holds SQLx, SMTP and crypto mechanics.
- `handler/httpapi` maps HTTP to use cases. It does not contain business rules.
- `xmodule` is the composition root. It is the only module that knows concrete adapters.

The React side is a single-page application. Vite serves the development build and proxies `/api` to Axum. The release binary serves `web/dist` and falls back to `index.html` for client routes.

## Requirements

- Rust 1.96 or newer
- Node.js 24 or newer and npm 11 or newer
- PostgreSQL 18 for local development, or Docker
- `openssl` for the setup script

## Quick start

```bash
make setup   # create .env, install web deps, migrate and seed
make dev     # API on :3000, Vite on :5173
```

Open http://localhost:5173. Vite proxies the API to Axum. The production build is served by Axum on port 3000. If port 3000 is busy, run `API_PORT=3100 make dev`.

`make setup` starts Postgres and Mailpit with Docker when no local Postgres listens on port 5432. If you use another Postgres instance, change `DATABASE_URL` in `.env`.

The container stack:

```bash
docker compose up --build
```

- Landing: http://localhost:3000
- Dashboard after sign-in: http://localhost:3000/dashboard
- Mailpit: http://localhost:8025
- Postgres: `127.0.0.1:5432` (Compose only)

The Compose stack runs the migrations and the seed before the app starts.

## Demo accounts

All demo accounts use the password `password`.

| Account      | Email                    | Role         | Can do                            |
| ------------ | ------------------------ | ------------ | --------------------------------- |
| Super Admin  | `superadmin@example.com` | `superadmin` | Everything                        |
| HR Staff     | `hr@example.com`         | `hr`         | Manage users, cannot manage roles |
| Regular User | `user@example.com`       | `user`       | Self-service only                 |

The seed creates eleven more users, including pending and suspended accounts. Sign in as `nabila@example.com` to see a blocked pending account. Sign in as `hendra@example.com` to see a blocked suspended account.

## Scripts

| Command                                      | Purpose                                            |
| -------------------------------------------- | -------------------------------------------------- |
| `make dev`                                   | Run the API and Vite together                      |
| `make api`                                   | Run only the Axum API                              |
| `make web`                                   | Run only Vite                                      |
| `make build`                                 | Build the web app and the release binary           |
| `make start`                                 | Run the release binary                             |
| `make lint`                                  | rustfmt, Clippy, ESLint, i18n parity and TypeScript |
| `make test`                                  | Cargo and Vitest                                   |
| `make test-e2e`                              | Playwright against the test database               |
| `make db-migrate`                            | Apply the embedded migrations                      |
| `make db-seed`                               | Seed permissions, roles, settings and demo users   |
| `make db-reset`                              | Drop and rebuild the development schema            |
| `make themes`                                | Regenerate themes and theme options                |
| `make rename NAME="Client App" SLUG=client-app` | Rename the project identity                     |

## RBAC wiring

- Role and permission definitions live in `api/src/entity/role.rs` and `api/src/entity/permission.rs`.
- Database rows live in `roles`, `permissions` and `role_permissions`. A user has one role.
- Coarse gate: `CurrentSession` in a handler, then `Actor::require(Permission::ManageUsers)` in the use case.
- Fine gate: policies in `api/src/entity/authorization.rs` (`can_manage_user`, `can_delete_user`, `can_assign_role`, `can_manage_role`).
- Navigation gating: `permissions` and `roles` arrays in `web/src/config/navigation.ts`.
- Privilege escalation is blocked in the use cases. Only actors with `manage_roles` can assign `superadmin`. The last superadmin cannot be removed.
- The superadmin role is immutable. The default `user` role can change its label, description and permissions, but not its name.

## Auth and approval

- Axum owns credentials, sessions, verification, reset and 2FA.
- Passwords use Argon2id. Session tokens are random 256-bit values. The database stores only a SHA-256 hash of each token.
- Registration uses an explicit sign-in after sign-up. The form reads the new user's status:
  - `active` -> sign in and go to the dashboard.
  - `pending` -> go to login with an approval message.
- The sign-in use case blocks pending and suspended users.
- The approval switch on the Users page writes `settings.registration_approval`.
- Sensitive actions can require a recent password confirmation. The signed cookie lasts 15 minutes and is used by account deletion.

## i18n

- Messages: `web/src/messages/en.json` and `web/src/messages/id.json`.
- The `locale` cookie is the single source of truth. There is no `/[locale]` URL segment.
- `web/src/lib/i18n.tsx` reads the cookie and loads the catalogue.
- `npm run check:i18n` fails when the catalogues drift.
- Zod messages are translation keys (`validation.*`). Forms translate them with `translateFieldError`.

## API overview

The full list is in `docs/routes-reference.md`. The main groups:

- `/api/auth/*` for sign-up, sign-in, sessions, password, email verification, confirmation and 2FA.
- `/api/users*` for user management and the registration approval switch.
- `/api/roles*` for roles and the permission matrix.
- `/api/settings/*` for profile and account deletion.
- `/api/dashboard` for the admin aggregate.
- `/api/health` for the container health check.

## Where things live

| What                   | Where                                                              |
| ---------------------- | ------------------------------------------------------------------ |
| API entry point        | `api/src/main.rs`                                                  |
| Domain entities        | `api/src/entity/`                                                  |
| Driven ports           | `api/src/app/port/`                                                |
| Use cases              | `api/src/app/usecase/`                                             |
| SQLx, SMTP and crypto  | `api/src/adapter/`                                                 |
| HTTP handlers          | `api/src/handler/httpapi/`                                         |
| Composition root       | `api/src/xmodule/`                                                 |
| Migrations             | `api/migrations/`                                                  |
| Seed                   | `api/src/adapter/postgres/seed.rs`                                 |
| React entry point      | `web/src/main.tsx`                                                 |
| Routes                 | `web/src/App.tsx`                                                  |
| Pages                  | `web/src/pages/`                                                   |
| App shell              | `web/src/components/app-shell/`                                    |
| UI primitives          | `web/src/components/ui/`                                           |
| Theme picker           | `web/src/components/theme/`, `web/src/lib/theme.ts`                |
| Navigation             | `web/src/config/navigation.ts`                                     |
| Messages               | `web/src/messages/`                                                |
| Design contract        | `DESIGN.md`                                                        |
| Architecture decisions | `docs/adr/`                                                        |
| Product docs           | `docs/`                                                            |
| Agent skill            | `.agents/skills/rust-skills/`                                      |

## Testing

```bash
make test       # Cargo unit tests and Vitest
make test-e2e   # Playwright, uses rust_boilerplate_test
```

The Rust integration test reads `TEST_DATABASE_URL` and skips when it is not set. Playwright starts the API on port 3100 with the test database and Vite on port 5173. It migrates and seeds the test database before the suite.

## Docker and deployment

- The Dockerfile builds the React app, the release binary and a non-root runtime image.
- The Compose stack includes Postgres, Mailpit, a one-shot migrate service, a one-shot seed service and the app.
- Set `APP_SECRET`, `APP_URL`, `WEB_URL`, `DATABASE_URL`, `APP_NAME` and `APP_LOCALE` in the deployment environment. `WEB_URL` is the public web origin used in email links. It defaults to `APP_URL`. In Compose, set `PUBLIC_WEB_URL` to override it without changing the development value in `.env`.
- The app listens on `0.0.0.0:3000` inside the container. The Compose file publishes it on loopback by default.
- Set `ALLOWED_ORIGINS` when a separate origin serves the web app. The JSON API uses `SameSite=Lax` cookies and an Origin check.

## Agent docs

- `AGENTS.md` is the working guide for agents.
- `docs/architecture.md` explains the layers and request flow.
- `docs/auth-and-rbac.md`, `docs/users-and-roles.md` and `docs/settings.md` describe the product.
- `docs/adr/` records the cross-cutting decisions.
- `.agents/skills/rust-skills/` contains the Rust coding rules used for this port.

## Not included

Employee documents, attendance, leave requests, billing and deployment pipelines are intentionally absent. Add your own modules on both sides of the API.

## Docs

- `docs/architecture.md`
- `docs/auth-and-rbac.md`
- `docs/users-and-roles.md`
- `docs/settings.md`
- `docs/internationalization.md`
- `docs/routes-reference.md`
- `docs/demo-guide.md`
- `docs/agent-skills.md`
- `docs/adr/README.md`
- `AGENTS.md`
- `DESIGN.md`

## License

MIT.
