# ADR-0014: Evidence for canonical validation of slot acceptance

- Status: Accepted
- Date: 2026-09-25
- Related: workshop-rs #283, #21; wrightkit/opy-rs #366, #370; ADR-0001;
  ADR-0011

## Context

`validate_canonical_ids` is used as a hard compilation gate by `opy-rs` and
`del-rs`. Besides identity, arity, and enum-domain checks, it rejects a value
whose semantic type does not match the declared parameter type. Those declared
types come from community documentation (Workshop.codes articles) cross-checked
against upstream compiler data.

A builtin probe against pinned OverPy 9.7.10 found 111 programs that OverPy
compiles but canonical validation rejects, in three families:

1. non-Boolean values in Boolean parameters;
2. arrays in parameters declared as `Object` (`logToInspector`, `bigMessage`,
   `smallMessage`, `setObjectiveDescription`, `progressBarHud`);
3. `Number 0` in the `createEffect` position parameter, produced by folding a
   vector `x - x`.

The authority for what a Workshop program may contain is the live Overwatch
client, but the client cannot be driven in development and has no API.
[ADR-0005](0005-seasonal-client-validation.md) defines a capture workflow for
whoever has a client; it cannot be a precondition for ordinary changes. The
available evidence is community material: Workshop.codes, upstream compilers
(OverPy, OSTW), and Workshop text exported from real projects. These sources
prove different things, and a rule is needed for when each may relax or add a
rejection.

Two properties of the upstream compilers matter:

- Runtime equivalence does not imply import acceptance. The client
  type-checks pasted Workshop text by parameter: OverPy reported that the client
  rejects `Throttle Of` inside `Or` although a vector is a valid truth value at
  runtime ([Zezombye/overpy#182](https://github.com/Zezombye/overpy/issues/182)).
- OverPy emits a user-written value unchanged even when its own type check
  fails: the compiler hides `w_type_check` warnings by default
  ([`src/utils/logging.ts`](https://github.com/Zezombye/overpy/blob/1e2688954302a402d076944b46db07efb14d7b61/src/utils/logging.ts))
  and only rejects enum and constant mismatches. Passing a value through
  unchanged does not show that the client accepts it.

## Decision

1. **Ownership.** Which values a Workshop parameter accepts is canonical
   Workshop semantics owned by `workshop-rs`. There is one validation contract.
   No lenient or provider-specific entry point is added, and consumers do not
   carry private relaxations of Workshop acceptance.
2. **Validation contract.** Structural failures (unknown identities, arity,
   enum domain, and reference category) are always rejected. A type mismatch
   against a declared parameter type is a presumption: it is rejected until
   acceptance evidence for that form exists, and is then accepted. A rejection
   stricter than the declared type requires rejection evidence. When validation
   accepts a program, that does not claim the live client will import or run it.
3. **Evidence classes.**

   | Source | Proves | Scope |
   | --- | --- | --- |
   | Workshop text exported by the client in a real project, with immutable provenance ([ADR-0004](0004-real-project-evidence.md)) | acceptance | the observed parameter |
   | A deliberate upstream emission rule: an optimizer substitution (such as `0`/`False`/`Null`, `1`/`True`, or a zero vector/`Null`, or a custom colour replaced by a colour constant) or an emitter rewrite such as OverPy's `First Of` wrapping | acceptance of the emitted form, and rejection of the form the rule deliberately avoids | the scope the rule is keyed on: a per-parameter flag covers only that parameter; a rule keyed on a parameter type covers that type |
   | An upstream report of client behavior or a workaround for it | rejection, or acceptance where stated | the reported scope |
   | A declared type in Workshop.codes or upstream data | presumption only | — |
   | Upstream pass-through of a user-written value; a loosely declared upstream `any`; runtime-equivalence reasoning | nothing | — |
   | A live-client capture ([ADR-0005](0005-seasonal-client-validation.md)) | acceptance or rejection; overrides the classes above when they conflict | the captured case |

   Upstream data tables are cross-checks, not catalog sources
   ([ADR-0001](0001-catalog-boundaries.md)). A fact derived from upstream
   behavior is established by observing the pinned reference output, and its
   source is cited.
4. **Placement.** Evidence scoped to one parameter becomes a positional catalog
   fact ([ADR-0011](0011-contextual-semantic-placement.md)). Evidence scoped to
   a parameter type becomes a typed Rust rule. Each accepted or rejected branch
   records its evidence in the catalog source attribution and is protected by a
   regression at a valid structural context.
5. **Application to the probe findings.**
   - *Boolean parameters accept any value.* OverPy's emitter wraps a value in
     `First Of` only for a Boolean parameter whose value the client rejects, and
     is keyed on the parameter type
     ([`src/compiler/astToWorkshop.ts`](https://github.com/Zezombye/overpy/blob/1e2688954302a402d076944b46db07efb14d7b61/src/compiler/astToWorkshop.ts)).
     The fix for #182 states that all values can be put in Boolean fields and
     that values the client does not accept are wrapped. This is a typed rule
     for the `Boolean` parameter type. The set of values the client rejects in
     that position is not modelled; it may be added later as rejection
     evidence observed from the pinned wrapping behavior or a client capture.
   - *Arrays in the five `Object` text parameters stay rejected.* Workshop.codes
     declares these parameters as `Object` and names `Array` separately where it
     is accepted, OverPy declares `Object`, OSTW's `any`
     ([`Elements.json`](https://github.com/ItsDeltin/Overwatch-Script-To-Workshop/blob/dde3da1e99d9cfc60ab7de4c2e6c0fa5849108a9/Deltinteger/Deltinteger/Elements.json))
     is loose typing, and OverPy passes such arrays through without an emission
     rule. No acceptance evidence exists. `opy-rs` records the difference as an
     approved exception, with a pinning test that native compilation rejects it.
   - *`Number 0` in the `createEffect` position stays rejected.* The `0` is a
     constant-folding artifact, not a substitution OverPy declares for that
     parameter. `opy-rs` records an approved exception in the same way.
   - An `optimizeForSize` substitution that the pinned reference writes into a
     parameter lacking the matching catalog coercion (such as `createDummy`) is
     per-parameter acceptance evidence and becomes a positional catalog fact.

## Alternatives considered

- **A separate lenient validation entry point for `opy-rs`.** Rejected: its
  leniency would be defined by what OverPy writes, which places provider
  behavior in the Workshop core. It also leaves `del-rs` and other consumers
  with the same false positives, and gives the repository two validation
  contracts.
- **Approved exceptions in `opy-rs` for all three families.** Rejected: the
  Boolean family is a validation defect with documented evidence, not an
  upstream deviation.
- **Relaxing all three families together.** Rejected: families 2 and 3 have no
  acceptance evidence.
- **Deriving acceptance from runtime equivalence of values.** Rejected: import
  acceptance is a separate client check, as #182 shows.

## Consequences

- Validation rejects fewer programs that the client is documented to accept,
  and may accept some programs the client rejects (for example a vector in a
  Boolean parameter) until rejection evidence is recorded. Diagnostic accuracy
  for such forms is left to evidence-backed rules rather than declared types.
- Existing declared-type checks remain until conflicting acceptance evidence
  appears; they are not removed in bulk.
- `opy-rs` owns the `First Of` wrapping and the two approved exceptions as
  OverPy structural behavior.
- A client capture can later confirm, tighten, or relax any of these facts
  without changing the contract.
