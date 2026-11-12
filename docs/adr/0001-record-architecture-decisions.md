# 0001 — Record architecture decisions

- Status: accepted
- Date: 2026-11-12

## Context

The boilerplate makes choices that are cheap to reverse in code and expensive to reverse after a project builds on them: the HTTP framework, the data layer, the auth model, the locale strategy and the UI layer.

Future agents and developers need the reason, not only the code.

## Decision

Record cross-cutting decisions as ADRs in `docs/adr/`. Each record states the context, the decision, the alternatives and the consequences. Supersede an old record with a new one. Do not rewrite the old one.

## Consequences

- New decisions are reviewable before implementation.
- The repository carries a small documentation cost.
- The index in `docs/adr/README.md` changes with each record.
