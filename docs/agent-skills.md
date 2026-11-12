# Agent skills

The repository ships one agent skill:

| Skill        | Path                          | Source                         | License |
| ------------ | ----------------------------- | ------------------------------ | ------- |
| `rust-skills` | `.agents/skills/rust-skills/` | Local Rust rules used for this port | MIT     |

The skill is a set of rule files across 26 categories: ownership, error handling, async, concurrency, unsafe code, API design, testing, observability and project structure. Agents should read the relevant rule file before they change Rust code.

## How to use

1. Read `AGENTS.md`.
2. For Rust work, open the rule category that matches the task. For example:
   - new async handler: `async-`, `err-`, `own-`
   - new public type: `api-`, `type-`, `doc-`
   - SQL or adapter work: `err-`, `serde-`, `test-`
3. Apply the rules. The repository runs `cargo fmt`, `cargo clippy -- -D warnings` and `cargo test`.
4. For frontend work, read `DESIGN.md` and the rules in `AGENTS.md`. The React stack has no vendored skill in this repository.

## Why only one skill

The Next.js sibling committed vendor skills for Next.js, Prisma, Better Auth, shadcn/ui, Docker and Playwright. Those skills describe tools this port does not use. Shipping them would give agents stale guidance.

The React, Vite, TanStack Query and shadcn/ui APIs are covered by the code and `DESIGN.md`. Add a vendor skill only when the vendor publishes one and the repository documents the source, version and license.

## Updating

Replace `.agents/skills/rust-skills/` with a newer MIT-licensed copy. Keep `docs/agent-skills.md` and `AGENTS.md` in sync.

Do not copy an unlicensed skill into the repository. Community skills need a review and a documented reason.

## Layout

```text
.agents/
  skills/
    rust-skills/
      SKILL.md
      rules/
        own-borrow-over-clone.md
        err-thiserror-lib.md
        async-no-lock-await.md
        ...
```

The rule identifiers are stable. Cite them in reviews when a change follows or deliberately breaks a rule.
