# 0011 — Hexagonal Rust layers

- Status: accepted
- Date: 2026-11-13

## Context

The Rust API started as a small Axum app. A flat layout would put SQL, auth rules and HTTP mapping in the same modules. The team already uses hexagonal architecture in `manus-dei`, with `entity`, `app/port`, `app/usecase`, `adapter`, `handler` and `xmodule`.

## Decision

Use the same layer flow in Rust:

```text
handler/httpapi -> app/usecase -> app/port <- adapter/*
                       |
                    entity
```

- `entity` is pure.
- `app/port` owns the traits. `async-trait` is used so the traits stay object-safe.
- `app/usecase` depends on entities and ports.
- `adapter` implements the ports and converts rows to entities.
- `handler/httpapi` is the driving adapter.
- `xmodule` is the composition root.

## Consequences

- Use cases are testable with fakes at port boundaries.
- SQL and SMTP changes do not touch business rules.
- The repository has more files than a flat Axum app, but each file has one job.
