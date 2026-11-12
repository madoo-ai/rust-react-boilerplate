# 0003 — SQLx and PostgreSQL

- Status: accepted
- Date: 2026-11-12

## Context

The source uses Prisma 7. A Rust port needs async SQL, migrations, a seed and control over transactions.

## Decision

Use PostgreSQL 18 and SQLx 0.9.

- Postgres is the only database path.
- Migrations are embedded with `sqlx::migrate!` and committed under `api/migrations`.
- Queries are runtime-checked with bound parameters. Dynamic filters use `QueryBuilder`.
- The SQLx row types stay inside `adapter/postgres`. They convert to entities at the boundary.
- IDs are ULIDs stored as text. Timestamps are `timestamptz`.
- The seed is idempotent.

## Alternatives

- Diesel: synchronous by default and a different migration flow.
- SeaORM: an entity model on top of SQLx. The boilerplate is small enough for explicit SQL.
- Prisma with a Rust client: no first-party Rust client.

## Consequences

- The adapter is explicit and testable.
- Schema changes need a migration and a seed update.
- The release binary needs no separate query engine.
