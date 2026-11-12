# 0008 — Boilerplate is a scaffold

- Status: accepted
- Date: 2026-11-12

## Context

A boilerplate that tries to be every product becomes a maintenance burden and forces unrelated features into the next project.

## Decision

Treat this repository as a scaffold.

- Keep the core: auth, approval, RBAC, users, roles, settings, i18n, theme and the app shell.
- Leave domain modules (documents, attendance, leave requests, billing, inventory) out.
- Replace models, routes, use cases, navigation and tests when a project needs a different domain.
- Replace `DESIGN.md`, `web/src/styles/globals.css` and the shared components when a project needs a different visual identity.
- Keep the quality gates green after the shape changes.

## Consequences

- The next project starts with a small base.
- Removing a feature is a normal operation.
- `AGENTS.md` tells agents to rework or remove what the product does not need.
