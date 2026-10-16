# Lessons

Short review queue of reusable failure classes from merged workshop-rs work. Each entry names a prevention action and the evidence behind it. Promote an entry into a regression test, an owner contract, or an existing engineering standard, then remove it here. The procedure is the `wrightkit-retro` skill in `wrightkit/.agents`.

### A sibling path must not enforce a weaker subset of the same contract
- **Rule:** When adding a second path that faces the same contract — a tolerant carry, a new validation entry point, `source_edit` beside `write`, CLI beside library, a scoped beside an unscoped query — reuse the established path's check or enumerate its obligations. A new path must not accept what its sibling rejects, nor produce what its sibling cannot consume.
- **Why:** Every pairing drifted the same way: signature rendering spelled enum defaults differently than the emitter, scoped lookup ignored locale and scope validity, verbatim carriage silently accepted misspelled declared members, catalogued-key blocks were rejected on all three layers while uncatalogued siblings carried, emission re-validated pass-through members check had accepted, `source_edit` skipped `write`'s leaf rejection, `check` admitted hero members `emit` cannot emit, and the CLI skipped canonical-ID validation. Each fix converged on sharing the sibling's check.
- **Prevention home:** regression tests asserting sibling paths agree on shared inputs; owner contract in `AGENTS.md`.
- **Incidents:** [#378](https://github.com/wrightkit/workshop-rs/pull/378), [#386](https://github.com/wrightkit/workshop-rs/pull/386), [#406](https://github.com/wrightkit/workshop-rs/pull/406), [#413](https://github.com/wrightkit/workshop-rs/pull/413), [#424](https://github.com/wrightkit/workshop-rs/pull/424), [#435](https://github.com/wrightkit/workshop-rs/pull/435), [#438](https://github.com/wrightkit/workshop-rs/pull/438), [#439](https://github.com/wrightkit/workshop-rs/pull/439) — observed 2026-10-05, 2026-10-06, 2026-10-09, 2026-10-11, 2026-10-13

### Every declared catalog surface must parse back
- **Rule:** Spellings, aliases, and signatures declared in the catalog must all be exercised through the frontend, not sampled. When a declared surface does not parse, decide explicitly whether the declaration or the parser is wrong; do not leave the gap for consumers to find.
- **Why:** Consumers writing exactly what the catalog declares hit parse failures the suite never exercised: six colon-containing enum spellings rejected in bare position, index-action signatures omitting the surface player argument, and declared localized spellings failing to reparse in 11 of 15 locales.
- **Prevention home:** census/exhaustive catalog tests — the localization shard now enumerates every declared spelling ([#449](https://github.com/wrightkit/workshop-rs/pull/449)); signatures and colon-shape members need the same treatment.
- **Incidents:** [#443](https://github.com/wrightkit/workshop-rs/pull/443), [#445](https://github.com/wrightkit/workshop-rs/pull/445), [#449](https://github.com/wrightkit/workshop-rs/pull/449) — observed 2026-10-14

### Emitted text must reparse to the same program
- **Rule:** Round-trip coverage must feed emitter output back through the parser, including edge shapes — empty members, escapes, BOM, duplicate blocks, non-finite and out-of-range literals. Asserting the emitted string alone does not catch output the frontend reads differently.
- **Why:** Emit/parse drift shipped repeatedly: a trailing space on empty verbatim members, a bare `name:` member absorbing the next line (the existing round-trip test never re-parsed), and the audit fidelity batch — rule-name escapes, truncating declaration indexes, silently dropped duplicate `settings` blocks, opaque-string escapes, and a lexer that rejected BOM.
- **Prevention home:** existing policy — `wrightkit/.github` `docs/testing-policy.md` lists "parse -> emit -> parse preserving declared semantics"; repo roundtrip regression tests ([#437](https://github.com/wrightkit/workshop-rs/pull/437)).
- **Incidents:** [#398](https://github.com/wrightkit/workshop-rs/pull/398), [#404](https://github.com/wrightkit/workshop-rs/pull/404), [#433](https://github.com/wrightkit/workshop-rs/pull/433), [#436](https://github.com/wrightkit/workshop-rs/pull/436), [#437](https://github.com/wrightkit/workshop-rs/pull/437) — observed 2026-10-09, 2026-10-13

### Retained-source bookkeeping must audit only the attached file's spans
- **Rule:** When attaching source text to a registered file, prune only spans that belong to that file and fail to resolve in it. Provider maps legitimately record spans outside authored extents (macro expansion onto invocation sites); cover multi-file attach order in tests.
- **Why:** The first fix assumed every recorded provenance span resolves in the attached text; the follow-up found the audit ran against every file's spans, clearing earlier files' provenance when sources attach sequentially — multi-file provider programs lost all provenance.
- **Prevention home:** regression tests for multi-file attach ordering and non-resolving provider spans.
- **Incidents:** [#410](https://github.com/wrightkit/workshop-rs/pull/410), [#418](https://github.com/wrightkit/workshop-rs/pull/418) — observed 2026-10-09

### New syntax must fit the pinned MSRV, not just stable
- **Rule:** `rust-version = "1.85.0"` gates syntax, not only APIs — let-chains and newer constructs fail the 1.85 CI job even when stable accepts them. Check the MSRV toolchain (or enable `clippy::incompatible_msrv` against `rust-version`) before pushing.
- **Why:** Two PRs needed in-flight commits dropping let-chains after `Rust (1.85.0) / Quality` failed while `stable` passed — the same author-side mistake recurred within two days.
- **Prevention home:** lint/CI — enable `clippy::incompatible_msrv` so local `cargo clippy` catches post-MSRV syntax, or document the `+1.85.0` check in `AGENTS.md`.
- **Incidents:** [#433](https://github.com/wrightkit/workshop-rs/pull/433), [#449](https://github.com/wrightkit/workshop-rs/pull/449) — observed 2026-10-13, 2026-10-14

### A contract conflict is resolved by its owner, not by the PR that raises it
- **Rule:** When a change conflicts with an architecture or public-contract rule, the deciding ADR must carry the owning role's approval before it is marked `Accepted`; the implementation PR cannot self-authorize the decision it needs.
- **Why:** The residual-tolerant validation PR authored ADR-0018 as `Accepted` to resolve a contract conflict its own change created; the decision only landed after the owner reviewed and approved it, costing three review rounds.
- **Prevention home:** existing policy — workspace `AGENTS.md` "Role and self-authorization", `entropy-policy.md`, `pr-review.md`.
- **Incidents:** [#373](https://github.com/wrightkit/workshop-rs/pull/373) — observed 2026-10-05
