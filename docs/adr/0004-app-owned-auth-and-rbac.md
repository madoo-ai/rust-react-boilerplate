# 0004 — App-owned authentication and RBAC

- Status: accepted
- Date: 2026-11-12

## Context

The source uses Better Auth for credentials, sessions, verification, reset and 2FA, and app-owned tables for roles and permissions. Rust has no drop-in equivalent with the same API.

## Decision

Own the auth flows and keep authorization in the domain.

- Passwords use Argon2id.
- Sessions are opaque random tokens. The database stores only a SHA-256 hash.
- Email verification and reset use one-time tokens in `verifications`.
- TOTP and backup codes live in `two_factors`. Backup codes are hashed.
- The confirm-password cookie is signed with `APP_SECRET`.
- Roles, permissions and role assignments are entities and PostgreSQL tables.
- Use cases enforce permissions and policies. Handlers never trust a client role.

## Alternatives

- A hosted auth provider: user data and sessions leave PostgreSQL.
- A Rust auth framework: none matches the required 2FA and approval behavior without similar work.
- JWT sessions: revocation and "block suspended users now" are harder than with database sessions.

## Consequences

- The auth code is ours to test and review.
- The schema is close to the source, so the product behavior stays the same.
- Security fixes ship with the app, not with a third-party runtime.
