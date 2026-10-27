# ADR-0019: One catalog id may live in two sections; lookups name the section

- Status: Accepted
- Date: 2026-10-27
- Related: [Issue #563](https://github.com/wrightkit/workshop-rs/issues/563),
  [Issue #553](https://github.com/wrightkit/workshop-rs/issues/553);
  [ADR-0001](0001-catalog-boundaries.md),
  [ADR-0008](0008-canonical-public-program-boundary.md)

## Context

The committed catalog declares `modifyGlobalVariable` under both `structural`
and `actions`, `modifyPlayerVariable` likewise, and every modify operator
(`add`, `subtract`, …, `removeFromArrayByIndex`) under both `values` and
`operators`. Until now, resolving such an id depended on a hard-coded
resolution order rather than on the caller stating which section it meant:

- `catalog/load.rs` `insert_param_display_names` collected the indexes of
  *every* entry carrying the id across all sections and applied each display
  name to all of them. The id, not the section, picked the target.
- `lookup.rs` `within_callable` tried `Kind::Action`, then `Kind::Value`,
  then `Kind::Structural` for every scope, so `lookup_within(..,
  "modifyGlobalVariable", ..)` answered the `actions` entry while
  `paramDisplayNames` data targets the signature the `structural`
  declaration describes.
- `rules/statement.rs` carried two places where the code and the document
  disagreed about which section an id names. `action_call` kept a
  player-variable chase under the global chase's name (`chaseAtRate` with a
  `playerVariable` first argument stood in for the document's own
  `chasePlayerVariableAtRate` id), and `value_call` told a modify operand
  from a same-named value only by whether the node carried arguments.

These were split from #553 as items sharing one root cause: the catalog
identity model, a public contract of `workshop-rs`, had no answer to "the
same id in two sections".

## Decision

Owner decision (2026-10-27), implemented here:

1. One catalog id may be declared in more than one section. Ids are not
   renamed: `modifyGlobalVariable` stays `modifyGlobalVariable` in both
   `structural` and `actions`.
2. A lookup states the section it wants explicitly. There is no
   section-agnostic resolution of a dual-declared id.
   - `insert_param_display_names` resolves `paramDisplayNameGaps`
     `entryId[index]` positions and `paramDisplayNames` records against the
     `structural` section — the section whose signature and editor labels
     the recorded parameter names describe. A gap or display-name record
     that names only a non-structural id is still an error, so records can
     never silently target the wrong section.
   - `within_callable` in `lookup.rs` resolves a callable scope by trying
     sections in an explicit, fixed order — action, then value, then
     structural — and `Catalog::entry(kind, id)`/`Catalog::resolve` already
     take `kind` as the explicit section argument. `Catalog::lookup` already
     returns one `LookupMatch::Builtin` per catalog entry, so a
     dual-declared id reports both sections' matches.
   - `rules/statement.rs` `action_call` keeps a call node named by the
     document's own id: `chasePlayerVariableAtRate` /
     `chasePlayerVariableOverTime` convert to `Action::Call` under that id,
     never under the global chase's name, and the parser's folded form
     canonicalizes to the same id. A `chaseAtRate`/`chaseOverTime`
     `Action::Call` built with a `playerVariable` first argument — a model
     `Action::call` and the emitter still accept — renders the player id,
     matching the emission of the same model, so the produced document
     stays convertible. `value_call` tells a modify operand from
     a same-named value by section, not by argument count: an id that is a
     declared modify operation (`ModifyOp::from_catalog_id`) is an operand
     only when it carries no arguments, and the operand's id must be one
     the enclosing modify parameter's `paramContent.operations` accepts —
     an operation id the catalog declares as a `values` entry too is never
     read as a value call at an operand position, and vice versa.
3. Existing accepted lookup behavior stays unchanged: the
   `actions` answer still wins the unscoped `within` order, emitted text is
   byte-identical, and every existing catalog, lookup, parser, emission,
   and rule-content test keeps passing.

## Alternatives considered

- **Rename the duplicated ids** (for example a distinct structural id).
  Rejected by the owner decision: ids are not renamed. A rename would also
  change every emitted document and spelling that already resolves the id.
- **Keep section-agnostic resolution and let argument shape disambiguate
  everything.** Rejected: it makes the accepted vocabulary depend on node
  shape rather than on declared identity, which is the ambiguity the issue
  records.
- **Let a `paramDisplayNames` record apply to every section's entry of the
  id.** That was the load-time behavior being replaced; it is rejected
  because a record written for the structural signature could silently
  attach to an action entry whose parameter list differs.

## Acceptance criteria

1. A `paramDisplayNames` record or `paramDisplayNameGaps` position naming a
   dual-declared id resolves against the `structural` entry; a record for
   an id with no `structural` declaration resolves against the entry that
   declares it, and an id declared by no entry is still an error. Verified
   by catalog tests that load data whose records name a dual-declared id.
2. `Catalog::lookup` and `lookup_within` report the same matches as before
   for every existing query; the scoped `within` order (action, value,
   structural) is stated in code, not implied by iteration. Verified by the
   existing lookup tests plus a test naming a dual-declared id.
3. `statement_from_content` reads `chasePlayerVariableAtRate` /
   `chasePlayerVariableOverTime` and the modify operands by the section the
   node names; a `playerVariable` target under the global chase id and an
   operand id carrying arguments are refused. Verified by
   `statement_from_content` tests and by the rule-content round-trip tests
   (render → convert → render is unchanged).
4. `cargo test --workspace --all-targets` passes with no change to emitted
   text; the parser/emitter chase tests, the corpus round-trips, and the
   modify-operation tests are unmodified and green.

## Consequences

- `rule-content-v1` documents now mean what they say: a `call` id names the
  catalog entry of that id's section, and `chasePlayerVariableAtRate` is no
  longer aliased to `chaseAtRate` on read.
- The `Program` model for a player-variable chase changes from
  `Action::Call { name: "chaseAtRate", args: [PlayerVariable…, …] }` to
  `Action::Call { name: "chasePlayerVariableAtRate", args: [PlayerVariable…, …] }` — the name the document and the catalog both use. Emission of that
  model is unchanged because the emitter already maps the receiver-folded
  call back to the player-variable spelling.
- Downstream consumers that pattern-match `chaseAtRate` to detect a player
  chase must match both global and player ids; the model no longer hides
  the distinction.
- The catalog gains `paramCoercions` on `chasePlayerVariableAtRate` /
  `chasePlayerVariableOverTime` at the rate/duration parameter
  (`falseAsNumber`/`trueAsNumber`), so client-accepted `False`/`True`
  literals — the bastion.ow
  `Chase Player Variable At Rate(Event Player, hack_timer, False, True, None)`
  evidence — validate against the player's own entry now that the folded
  call resolves that id. The content digest is rebuilt
  (`1c31bdcb…` → `5eec00d3…`).
