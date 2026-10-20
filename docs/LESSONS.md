# Lessons

Short review queue of reusable failure classes from merged workshop-rs work. Each entry names a prevention action and the evidence behind it. Promote an entry into a regression test, an owner contract, or an existing engineering standard, then remove it here. The procedure is the `merged-pr-retro` skill in `wrightkit/.agents`.

### A contract conflict is resolved by its owner, not by the PR that raises it
- **Rule:** When a change conflicts with an architecture or public-contract rule, the deciding ADR must carry the owning role's approval before it is marked `Accepted`; the implementation PR cannot self-authorize the decision it needs.
- **Why:** The residual-tolerant validation PR authored ADR-0018 as `Accepted` to resolve a contract conflict its own change created; the decision only landed after the owner reviewed and approved it, costing three review rounds.
- **Prevention home:** existing policy — workspace `AGENTS.md` "Role and self-authorization", `entropy-policy.md`, `pr-review.md`.
- **Incidents:** [#373](https://github.com/wrightkit/workshop-rs/pull/373) — observed 2026-10-05

### A declared surface is derived from the model it describes, not enumerated from observed cases
- **Rule:** When a format field, lookup fact, or catalog declaration re-describes what the model accepts or produces, derive or check every declared fact against the model or the pinned export in the same change. Declared coverage written from the cases observed so far drifts from the cases that exist.
- **Why:** The lookup and rule-content-v1 surface shipped declared facts that disagreed with the model: a wrong `returnType` against the pinned export, 23 enum parameters with no `paramDomains`, an `Operation` domain three members wide where the model accepts eleven modify calls, lookup signatures that did not match rule-content arguments, and no content form for a parser-producible ambiguous member. A downstream consumer found each one — the same pattern as #452 on the previous declared surface.
- **Prevention home:** catalog-load validation and lookup↔content contract tests (landed in #526); the exhaustive-declaration decision pending in #452.
- **Incidents:** [#511](https://github.com/wrightkit/workshop-rs/pull/511) (fixes [#495](https://github.com/wrightkit/workshop-rs/issues/495)), [#512](https://github.com/wrightkit/workshop-rs/pull/512) (fixes [#505](https://github.com/wrightkit/workshop-rs/issues/505)), [#522](https://github.com/wrightkit/workshop-rs/pull/522) (fixes [#492](https://github.com/wrightkit/workshop-rs/issues/492)), [#526](https://github.com/wrightkit/workshop-rs/pull/526) (fixes [#503](https://github.com/wrightkit/workshop-rs/issues/503), [#504](https://github.com/wrightkit/workshop-rs/issues/504)) — observed 2026-10-19, 2026-10-20

### An additive enum variant goes at the end of the enum
- **Rule:** Insert a new enum variant after the existing variants; mid-enum insertion silently shifts the implicit discriminants of every variant below it and is a breaking API change.
- **Why:** A review finding caught a new `LocalizedString` variant inserted mid-enum shifting `Parameter` 4→5 and `SettingPath` 5→6; `cargo-semver-checks`' `enum_no_repr_variant_discriminant_changed` gates that as breaking.
- **Prevention home:** authoring review plus the `Public API compatibility` (cargo-semver-checks) CI job.
- **Incidents:** [#501](https://github.com/wrightkit/workshop-rs/pull/501) review finding — observed 2026-10-19

### Folded-length thresholds are re-derived per script, not inherited from Latin
- **Rule:** A minimum folded-length or character-count threshold must be checked against CJK input: characters carry more meaning there, so a cutoff tuned on Latin silently drops valid queries.
- **Why:** `Catalog::lookup` required a folded query of three characters before substring matching, so two-character CJK words that appear inside many display names (阈值, 变量) returned no entries.
- **Prevention home:** the two-character CJK regression landed in #508; authoring review of thresholds against multi-script input.
- **Incidents:** [#508](https://github.com/wrightkit/workshop-rs/pull/508) (fixes [#497](https://github.com/wrightkit/workshop-rs/issues/497)) — observed 2026-10-19
