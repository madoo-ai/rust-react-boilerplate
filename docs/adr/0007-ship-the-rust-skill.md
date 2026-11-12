# 0007 — Ship the Rust agent skill

- Status: accepted
- Date: 2026-11-13

## Context

Agents without current Rust rules fall back to stale habits: excessive clones, `unwrap` in production, locks across `.await`, and stringly typed APIs.

## Decision

Commit the MIT-licensed `rust-skills` rules under `.agents/skills/rust-skills/`.

- `docs/agent-skills.md` lists the source, license and usage.
- `AGENTS.md` tells agents which rule categories apply to which task.
- Do not copy the Next.js, Prisma or Better Auth vendor skills. This port does not use those tools.

## Consequences

- Agents get version-appropriate Rust guidance.
- The repository is larger by about 2 MB.
- A future update replaces the skill directory and updates the docs.
