# Lessons

Short review queue of reusable failure classes from merged workshop-rs work. Each entry names a prevention action and the evidence behind it. Promote an entry into a regression test, an owner contract, or an existing engineering standard, then remove it here. The procedure is the `wrightkit-retro` skill in `wrightkit/.agents`.

### A contract conflict is resolved by its owner, not by the PR that raises it
- **Rule:** When a change conflicts with an architecture or public-contract rule, the deciding ADR must carry the owning role's approval before it is marked `Accepted`; the implementation PR cannot self-authorize the decision it needs.
- **Why:** The residual-tolerant validation PR authored ADR-0018 as `Accepted` to resolve a contract conflict its own change created; the decision only landed after the owner reviewed and approved it, costing three review rounds.
- **Prevention home:** existing policy — workspace `AGENTS.md` "Role and self-authorization", `entropy-policy.md`, `pr-review.md`.
- **Incidents:** [#373](https://github.com/wrightkit/workshop-rs/pull/373) — observed 2026-10-05
