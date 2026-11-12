# Rust React Boilerplate — agent guide

This is a Rust + Axum + SQLx API with a React 19 single-page application. Treat the repository as the product, not as a collection of generated files.

## Boilerplate, not the product

This repository is a starting point, not the final application. It exists to give the next project a working auth, RBAC, localization, settings and design baseline.

If the requested product does not need a part of this boilerplate, rework or remove it. Do not keep a feature only because it is already here.

Before you delete, check:

- routes and layouts
- the database schema and seed data
- use cases, ports and HTTP handlers
- navigation and permission checks
- shared components and translations
- tests

After you change the shape, run `make lint`, `make test` and `make test-e2e`.

## Before you build

1. Read `README.md`, `DESIGN.md`, `docs/architecture.md`, `docs/adr/README.md` and this file.
2. Read the relevant Rust guide under `.agents/skills/rust-skills/` before you change Rust code.
3. Ask what is changing and who uses it.
4. For cross-cutting or irreversible decisions, add or supersede an ADR under `docs/adr/`.
5. If behavior changes, update the matching page under `docs/` in the same change.
6. Work in a small scope. Do not mix unrelated refactors with a feature.

## Map

| Need                               | Read                                                          |
| ---------------------------------- | ------------------------------------------------------------- |
| Product scope and setup            | `README.md`                                                   |
| Request lifecycle and conventions  | `docs/architecture.md`                                        |
| Auth, approval and RBAC flows      | `docs/auth-and-rbac.md`                                       |
| User and role management           | `docs/users-and-roles.md`                                     |
| Settings, 2FA and account deletion | `docs/settings.md`                                            |
| Locale system                      | `docs/internationalization.md`                                |
| Every route and gate               | `docs/routes-reference.md`                                    |
| Demo script                        | `docs/demo-guide.md`                                          |
| Architecture decisions             | `docs/adr/`                                                   |
| Visual rules                       | `DESIGN.md`                                                   |
| Domain entities and policies       | `api/src/entity/`                                             |
| Driven ports                       | `api/src/app/port/`                                           |
| Use cases                          | `api/src/app/usecase/`                                        |
| SQLx, SMTP and crypto adapters     | `api/src/adapter/`                                            |
| HTTP handlers                      | `api/src/handler/httpapi/`                                    |
| Composition root                   | `api/src/xmodule/`                                            |
| Migrations and seed                | `api/migrations/`, `api/src/adapter/postgres/seed.rs`         |
| React routes and pages             | `web/src/App.tsx`, `web/src/pages/`                           |
| App shell and landing              | `web/src/components/app-shell/`, `web/src/components/marketing/` |
| UI primitives                      | `web/src/components/ui/`                                      |
| Navigation                         | `web/src/config/navigation.ts`                                |
| Translations                       | `web/src/messages/{en,id}.json`                               |
| Tests                              | `api/tests/`, `web/tests/`                                    |
| Rust agent skill                   | `.agents/skills/rust-skills/`                                 |

## Commands

```sh
make setup                  # .env, web deps, local Postgres, migrate, seed
make dev                    # API on :3000 and Vite on :5173
make api                    # only the API
make web                    # only Vite
make test                   # Cargo and Vitest
make test-e2e               # Playwright on :5173, API on :3100
make lint                   # rustfmt, Clippy, ESLint, i18n, TypeScript
make fmt                    # rustfmt and Prettier
make build                  # web build and release binary
make db-migrate             # apply embedded migrations
make db-seed                # idempotent seed
make db-reset               # destructive reset of the dev database
docker compose up --build   # app, Postgres, Mailpit, migrate and seed
```

The development app listens on `http://localhost:5173`. The production binary listens on `http://localhost:3000`. Use `API_PORT=3100 make dev` if port 3000 is busy.

## Architecture rules

The Rust side is hexagonal. The dependency direction is:

```text
handler/httpapi -> app/usecase -> app/port <- adapter/*
                       |
                    entity
```

- `entity/` is pure Rust. It must not import `axum`, `sqlx`, `lettre` or any adapter.
- `app/port/` owns the traits. Adapters implement them. Keep the traits small.
- `app/usecase/` contains business logic and orchestration. It depends on entities and ports only. It must not import adapter packages.
- `handler/httpapi/` is the driving adapter. It parses HTTP, calls use cases and maps entities to JSON. It must not contain business rules.
- `adapter/` contains mechanics only: SQL, SMTP, crypto and clocks. SQL row types must not escape the adapter.
- `xmodule/` is the composition root. Handlers, use cases and adapters must not construct their own dependencies.
- `/api/*` routes always answer with JSON. Unknown API paths return the JSON 404 envelope, never the SPA.
- The API and the web app are separate deployables in development. The release binary serves `web/dist` with an SPA fallback.

## Auth and RBAC rules

- Authentication is app-owned. Sessions are opaque random tokens. Store only the SHA-256 hash.
- Passwords use Argon2id. Do not lower the hash cost to make tests faster.
- Session cookies are HTTP-only, SameSite=Lax and secure in production.
- A user has one role. `superadmin` is immutable. `user` is the default role and its name cannot change.
- Only actors with `manage_roles` may assign `superadmin`. The last superadmin cannot be deleted or demoted. HR cannot manage a superadmin.
- Pending or suspended users cannot create a session. The sign-in use case and the UI both block them.
- Registration approval is a database setting. Read it in the use case, not from a client value.
- Sensitive actions that need a recent password confirmation use the signed `pwd_confirmed` cookie.
- Mutating requests require a same-origin `Origin` header. Do not remove the middleware to make a client work.

## UI rules

- Build on `web/src/components/ui` (shadcn). Do not add a second component library.
- Icons come from `@x1nx3r/altics-react-icons` with named imports. Do not add `lucide-react`.
- Use semantic tokens (`bg-background`, `text-muted-foreground`, `border-border`), not raw palette classes.
- Use `FieldGroup` and `Field` for forms, `gap-*` instead of `space-y-*`, `size-*` for equal dimensions and `cn()` for conditional classes.
- Every page has one `h1`, rendered by `PageHeader` or the auth card.
- Pages use the full content width. Constrain the grid, not the page.
- The primary table sits in a bare `overflow-x-auto rounded-lg border` wrapper.
- Use the full `Card` composition.
- Overlays need a title. Use `AlertDialog` for destructive confirmation.
- Send feedback through Sonner toasts. Use `FormAlert` for form-level errors.
- Color themes are generated. Edit `web/src/lib/theme/opencode-themes.json` and run `npm run themes:generate`. Do not edit `web/src/styles/themes.css` or `web/src/lib/theme/theme-options.ts` by hand.
- Product screens use Inter. Marketing and auth headings use Instrument Serif.
- Motion is CSS-only and compositor-friendly. Animate `transform` and `opacity`. Gate it with `motion-safe` and keep the reduced-motion path usable.

## i18n rules

- Every user-facing string goes through `useTranslations`. Add keys to both `web/src/messages/en.json` and `web/src/messages/id.json`.
- The `locale` cookie is the source of truth. Do not add a `[locale]` route segment.
- Zod messages are translation keys (`validation.*`). Translate them in the form with `translateFieldError`.
- Run `npm run check:i18n` after you change messages.

## Testing rules

- Unit-test policies, hashing, tokens and pure helpers.
- Integration-test use cases against the test database. The test skips when `TEST_DATABASE_URL` is not set.
- E2E-test auth, RBAC, approval, user CRUD, roles and settings with Playwright. The suite uses `rust_boilerplate_test` and starts the API on port 3100.
- Do not disable security checks to make a test pass. Fix the flow.

## API rules

- Validate in the use case. Validation messages are i18n keys.
- Return the shared error envelope: `{ "error": { "code", "message", "fields"? } }`.
- Never return password hashes, session tokens or raw SQL errors.
- Revoke sessions when an account becomes inactive or a password is reset.
- Use ULIDs for primary keys. Use `timestamptz` for time.
- Add a migration for every schema change. Never edit an applied migration.
- Keep the seed idempotent.

## Agent skills

The Rust coding rules are committed under `.agents/skills/rust-skills/`. Read the relevant rule file before you change ownership, errors, async code, testing or API design.

Use the `find-skills` skill if the task needs a skill that is not in this repository. Do not copy an unlicensed skill into the repository.

## Demo accounts

All demo accounts use the password `password`.

| Role       | Email                    |
| ---------- | ------------------------ |
| superadmin | `superadmin@example.com` |
| hr         | `hr@example.com`         |
| user       | `user@example.com`       |
| pending    | `nabila@example.com`     |
| suspended  | `hendra@example.com`     |

The seed creates additional active users so the table and charts look real.
