# ADR-0016: Catalog content stays outside the Rust public API

- Status: Accepted
- Date: 2026-09-27
- Related: [Issue #311](https://github.com/wrightkit/workshop-rs/issues/311),
  [Issue #252](https://github.com/wrightkit/workshop-rs/issues/252),
  [Issue #299](https://github.com/wrightkit/workshop-rs/issues/299);
  [ADR-0001](0001-catalog-boundaries.md),
  [ADR-0008](0008-canonical-public-program-boundary.md)

## Context

[ADR-0008](0008-canonical-public-program-boundary.md) decision 5 generated a
typed inherent constructor on `Action` or `Value` for every catalog action and
value, named after the catalog id and taking one argument per catalog
parameter. At v0.9.1 that was 474 public methods (215 on `Action`, 259 on
`Value`). Each one forwarded to `Action::call` / `Value::call`, except that a
few values built a dedicated variant (`Value::Null`, `Value::EventPlayer`,
`Value::Vector`, `Value::Array`).

The method names and signatures came from catalog data, so catalog content was
part of the Rust semver surface. Removing or renaming an id, or adding a
parameter to an existing action, removed a public method or changed its
signature; #299 shipped as a breaking release for that reason. Catalog updates
follow Overwatch's seasonal cadence and need to ship as minor releases under
1.x.

No WrightKit consumer used the generated constructors. `opy-rs`, Wright, and
`deltin-rs` build catalog actions and values through `Action::call` /
`Value::call`.

## Decision

1. `workshop-rs` does not expose catalog-generated Rust items. The typed
   constructors are removed; this replaces ADR-0008 decision 5. The rest of
   ADR-0008 is unchanged.
2. Catalog actions and values are built with `Action::call` / `Value::call`
   from the canonical id and the arguments in catalog parameter order.
   Canonical validation against a catalog decides whether a call is valid.
   Hand-written constructors and the public `Action` / `Value` variants remain
   supported.
3. A catalog-only change (adding, removing, or renaming an entry or parameter)
   produces no Rust public-API difference. Its compatibility is a Workshop data
   question, reported through catalog identity and validation, not a
   `cargo semver-checks` question.

## Alternatives considered

- **Keep the constructors and ship catalog changes as breaking releases.**
  Rejected: every seasonal update would need a new major version under 1.x.
- **Keep the constructors behind `#[doc(hidden)]`.** Rejected: hidden items are
  still public Rust API, so the semver consequence is unchanged.
- **A catalog-versioned typed builder, or a separate typed-constructor crate.**
  Not decided here. It would be additive and needs its own decision.

## Consequences

- Catalog updates can ship as minor releases.
- Consumers get no compile-time check of catalog ids or parameter counts from
  the Rust API; canonical validation reports them instead.
- The build no longer runs a catalog code generator.
