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
and table order. An empty result reports that nothing matched.

Each match is a canonical construct the parser and emitter accept:

- `LookupMatch::Builtin` — a structural keyword, action, value, event, or
  operator, with its catalog `Kind`, canonical id, the requested locale's
  display name when mapped, and, for actions and values, a `Signature`.
- `LookupMatch::EnumDomain` — an enum domain and every member it accepts;
  this is the follow-up query for a parameter whose domain is too large to
  list inline in a signature.
- `LookupMatch::EnumMember` — a `Domain.Member` identity with its display
  name; canonical qualified queries such as `Hero.SOLDIER_76` resolve.
- `LookupMatch::Setting` — a `settings::SettingDefinition` projected from the
  reviewed settings table, carrying the valid key, path, scope, and value
  forms.

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
settings-table enum domain, an action or value id or spelling, then a
settings path prefix. `<team>`/`<hero>` template segments accept their
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

## Signature form

`Signature::text` renders the call syntax in the requested locale: parameters
in call order; a required parameter reads `name: Type`; a parameter with a
declared default reads `name=default`, or `name?` when the default is null;
an enum-typed parameter shows its domain, listing members inline once per
signature when the domain has at most 32 members and reporting the member
count for a larger domain. `Signature::params`, `required_params`,
`variadic`, `return_type`, and `domains` expose the same facts structurally.

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
