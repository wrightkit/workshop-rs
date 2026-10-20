# Canonical Workshop lookup

`Catalog::lookup` answers name and vocabulary questions in canonical Workshop
terms from the same catalog and settings table that drive parsing and
emission. Wright and `opy-rs` compose and present the results; the lookup owns
no data of its own, so results cannot drift from the accepted vocabulary.

## Query surface

```rust
use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::lookup::LookupMatch;

let catalog = Catalog::builtin()?;
let matches = catalog.lookup(&Locale::new("en-US"), "create hud txt")?;
# assert!(matches.iter().any(|m| matches!(m, LookupMatch::Builtin { id, .. } if id == "createHudText")));
# Ok::<(), workshop_rs::WorkshopError>(())
```

A query may be a display name in the requested locale, a primary-locale
display name, a canonical id, a near spelling or guess, or a settings path
prefix. Matches are ranked: exact spellings first, then folded-equal
spellings, near spellings, token prefixes, and substrings; ties keep catalog
and table order. Token-prefix and substring guesses apply only once the
folded query is long enough to be selective: three characters for
word-spaced scripts, two for scripts written without word spacing (CJK
ideographs, kana, hangul), where two characters already carry a whole word —
the zh-CN query `阈值` finds `开始定向阈值` and the other throttle names,
while a two-letter ASCII query still does not flood the results. An empty
result reports that nothing matched.

Each match is a canonical construct the parser and emitter accept:

- `LookupMatch::Builtin` — a structural keyword, action, value, event, or
  operator, with its catalog `Kind`, canonical id, the requested locale's
  display name when mapped, a localized `description` for actions and
  values, and, for callable builtins, a `Signature`.
  Actions and values are always callable; a structural construct is callable
  when it declares a parameter list (`setGlobalVariable`, `if`,
  `forPlayerVariable`, `callSubroutine`, … — `else` and `end` declare an
  empty one), while section keywords such as `rule`, `event`, and `actions`
  stay without a signature.
- `LookupMatch::EnumDomain` — an enum domain and every member it accepts;
  this is the follow-up query for a parameter whose domain is too large to
  list inline in a signature.
- `LookupMatch::EnumMember` — a `Domain.Member` identity with its display
  name; canonical qualified queries such as `Hero.SOLDIER_76` resolve.
- `LookupMatch::Setting` — a `settings::SettingDefinition` projected from the
  reviewed settings table, carrying the valid key, path, scope, and value
  forms.
- `LookupMatch::LocalizedString` — a preset string identity from the
  catalog's `localizedStrings` table, with the requested locale's display
  name when mapped. This is the id a `{"localizedString": id}` rule-content
  node or a `Value::LocalizedString` carries, so the literal's display
  spelling resolves like any other catalog identity.

A locale the catalog does not declare is an explicit `WorkshopError::Unsupported`;
the catalog never answers localized names it cannot attest.

## Scoped lookup

`Catalog::lookup_within` lists the entries an identity *contains*: the
members of an enum domain, the parameters of a callable, or the settings
keys and segments under a path prefix.

```rust
use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::lookup::LookupMatch;

let catalog = Catalog::builtin()?;
let members = catalog.lookup_within(&Locale::new("en-US"), "Team", None)?;
assert!(members.iter().all(|m| matches!(m, LookupMatch::EnumMember { domain, .. } if domain == "Team")));
# Ok::<(), workshop_rs::WorkshopError>(())
```

`within` resolves in a fixed order — a catalog enum domain, a
settings-table enum domain, a callable by id or spelling (an action, a
value, or a structural construct), then a settings path prefix.
`<team>`/`<hero>` template segments accept their
own template spelling or a canonical team/hero key (`heroes.team1`,
`heroes.<team>`), never an arbitrary value, so a literal path stays
literal and `heroes.not-a-team` is an unknown scope. Children keep the
scope's own order (domain order, call order, table order); a non-empty
`query` filters and ranks them with the same scoring an unscoped lookup
applies, so a scoped query and an unscoped one agree on which names
match. Member spellings in the requested locale match and surface the
same way catalog spellings do.

Scoped results add two child shapes an unscoped lookup never answers:
`LookupMatch::Parameter` — one parameter of the scoped callable with its
call-order position, required flag, and `SignatureParam` facts (and its
enum domain with members when enum-typed) — and `LookupMatch::SettingPath`,
the next path segment below a settings prefix (`heroes.<team>`,
`heroes.general`) where the prefix is not yet a leaf key. A `within`
naming no known scope is an explicit
`WorkshopError::Unknown` with `kind: "lookup scope"`.

## Listing entries

`Catalog::lookup_entries` enumerates the whole listable vocabulary of a
kind under `locale` — no query, matching, or ranking; the result keeps
catalog order. Each element is the same `LookupMatch` a `lookup` query
builds for that entry, signature included, so a consumer paging through
the list sees the entry a query would have returned.

```rust
use workshop_rs::catalog::{Catalog, Kind, Locale};
use workshop_rs::lookup::LookupMatch;

let catalog = Catalog::builtin()?;
let values = catalog.lookup_entries(&Locale::new("en-US"), Kind::Value)?;
assert!(values.iter().all(|m| matches!(m, LookupMatch::Builtin { kind: Kind::Value, .. })));
# Ok::<(), workshop_rs::WorkshopError>(())
```

`Kind::Action`, `Kind::Value`, and `Kind::Event` list
`LookupMatch::Builtin` matches in `entries_of` order; `Kind::Enum`
lists one `LookupMatch::EnumDomain` per declared domain in
`enum_domains` order. A domain's members and a callable's parameters
are scoped children that already list through `lookup_within`.
Structural keywords, operators, and settings have no listing surface —
naming such a kind is a `WorkshopError::Unsupported`, as is a locale
the catalog does not declare. The result is the complete vocabulary of
the kind — paging, cursors, and presentation belong to the consumer.

## Signature form

`Signature::text` renders the call syntax in the requested locale: parameters
in call order; a required parameter reads `name: Type`; a parameter with a
declared default reads `name=default`, or `name?` when the default is null;
an enum-typed parameter shows its domain, listing members inline once per
signature when the domain has at most 32 members and reporting the member
count for a larger domain. `Signature::params`, `required_params`,
`variadic`, `return_type`, and `domains` expose the same facts structurally.

Structural construct signatures name the arguments of the construct's
`rule-content-v1` call in argument order (`setGlobalVariable` reports
`variable`, `value`); a variable or subroutine parameter's type names the
format's tagged `{"variable"}`/`{"subroutine"}` node (`Variable`,
`Subroutine`). Zero-argument constructs (`else`, `end`) report an empty
`params`.

## Descriptions

Every action and value carries a localized description — the short
in-game-documentation text the Workshop editor shows — and every parameter
position carries its own. `LookupMatch::Builtin.description` and
`SignatureParam.description` expose them as `Option<Description>` alongside
the other entry facts; `Catalog::description` and
`Catalog::param_description` resolve the same texts by kind, id, and
position.

A `Description` is the resolved `text`, the `locale` that text is actually
written in, and a `fallback` flag: requesting a declared locale the dataset
ships no text for returns the primary locale's (`en-US`) text with
`fallback: true`, so a consumer can render or suppress the marker. A
position or entry on the dataset's `gaps` list resolves to `None` rather
than fabricating text — for example `customString`, whose upstream
documentation carries no real description.

Descriptions are catalog data, not Rust items
([ADR-0016](adr/0016-catalog-content-outside-rust-api.md)): they live in
`crates/workshop-rs/src/catalog/data/descriptions.json`, a separate dataset
indexed by canonical id and parameter position next to `catalog.json`, with
its own version, content digest, `locales`, `pendingLocales`, `gaps`, and
`provenance` records, validated against the catalog at load. `en-US` ships;
`zh-CN` is declared as a pending stage. The dataset cannot drift from the
catalog: for every shipped locale each action, each value, and each
parameter position must carry text or name a gap — never both, never
neither.

## Content-layout facts

The text signature follows the Workshop surface call syntax; several
variable-target actions lay their `rule-content-v1` arguments out
differently. `SignatureParam::content` carries the per-parameter content
facts the catalog declares at those positions (`paramContent`, a
per-position array parallel to `params`), so the emitted document's
argument layout is derivable from `lookup` alone. A parameter without a
`content` fact maps one-to-one to a plain content argument.

`fold` declares membership in a folded variable-target call: the parameter
supplies `args[arg]` of the reserved `globalVariable`/`playerVariable` call
named by `call`, and the folded call occupies the content-argument position
of the fold group's first member. All members of a group share `call`, are
contiguous parameters, and cover the call's arguments exactly once. The
member's node form follows its declared parameter type: a `Variable`
parameter supplies the tagged `{"variable": name}` node bound to the
declared variable name, and any other parameter supplies a normal value
node.

```text
setPlayerVariableAtIndex(player, variable, index, value)
  paramContent[0] = {"fold": {"call": "playerVariable", "arg": 0}}
  paramContent[1] = {"fold": {"call": "playerVariable", "arg": 1}}
  → emits {"call": "playerVariable", "args": [<player>, {"variable": <name>}]}
    as content argument 0; `index` and `value` follow as arguments 1 and 2.
```

The global chase entries (`chaseAtRate`, `chaseOverTime`) and the player
forms (`chasePlayerVariableAtRate`, `chasePlayerVariableOverTime`,
`stopChasingPlayerVariable`, `setPlayerVariableAtIndex`,
`modifyPlayerVariableAtIndex`) declare these facts, as do the global forms
(`stopChasingGlobalVariable`, `setGlobalVariableAtIndex`,
`modifyGlobalVariableAtIndex`). Entries whose `rule-content-v1` arguments
already match their parameter list — including the structural variable
calls, whose `Variable`/`Subroutine` parameters name the tagged nodes
directly — declare no `fold`.

## Settings vocabulary

Settings matches are `SettingDefinition` values from the reviewed settings
table: querying by display name (`"Score To Win"`) or by path prefix
(`"gamemodes.control"`, `"heroes.<team>.<hero>"` — template segments accept
concrete team and hero names) returns valid keys with their scope, target
kind, and value domain.

## Candidate-bearing diagnostics

Raw Workshop `unknown ... spelling` diagnostics carry `candidates`: the
nearest accepted spellings from the same canonical vocabulary the parse
surface checks, ranked by the shared comparison fold. A rejecting site
that computed candidates emits `WorkshopError::UnknownWithCandidates`
(`candidates()` returns the list on it, empty elsewhere); sites without
candidate context still emit `WorkshopError::Unknown`. The diagnostic
message names the candidates — `unknown action spelling 'Create HUD Txt'
for locale 'en-us' (did you mean 'Create HUD Text'?)` — so consumers that
only see the message text, including closed-schema wire diagnostics, still
receive them. The diagnostic's code, locale, and span are unchanged, and
`candidates` is empty when nothing is close.
