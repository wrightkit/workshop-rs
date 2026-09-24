# Workshop source preservation

`workshop-rs` keeps canonical Workshop meaning in the public `Program` model.
Authored source is optional metadata retained by raw parsing, so
programmatically constructed programs do not need synthetic source metadata.

## Supported source

`parser::parse` and `parser::parse_with_context` retain the exact input in a
`SourceDocument` attached to file 0. The parsed public `Program` exposes that
document through `Program::source`. A consumer constructing a program can use
`Program::add_file` with the source types directly when it has authored source
for a file. The document preserves every byte,
including whitespace and newlines, and indexes `//` line comments outside
string literals. Other trivia is intentionally not assigned semantic identity;
it remains available in the exact source text.

The normalized storage keeps optional `Span` values for source-aware internal
operations. A comment is
attached to a node only when its complete byte range is contained by that
node's span, through `SourceDocument::comments_for`. Comments outside a node
remain document-level source. No neighboring node is guessed as their owner.
This makes attachment deterministic for rules, actions, values, and settings
regions while keeping the file document the authority for the complete source.

## Source edits

`SourceDocument::edit` and `SourceDocument::edit_span` create checked
`SourceEdit` replacements. Applying an edit verifies the original bytes and
rejects stale, invalid, or overlapping edits. `SourceDocument::apply` returns a
new document with comments reindexed; consumers reparse its text to obtain new
semantic spans. Bytes outside an explicit edit are unchanged, so unrelated
comments, whitespace, and mixed source structure are preserved.

Parsed programs expose rule, condition, action, and direct action-argument
spans through `Program::rule_span`, `Program::condition_span`,
`Program::action_span`, and `Program::action_argument_span`. Consumers that
construct a program can attach the same metadata with
`Program::set_rule_span`, `Program::set_condition_span`,
`Program::set_action_span`, and `Program::set_action_argument_span`; declaration
spans use the corresponding variable and subroutine methods. `Program::edit_source`
creates a checked edit from those public spans without exposing normalized WIR
storage. Programmatic construction remains source-free when no files or spans
are attached.

The settings API's `SettingSourceEdit` remains the typed settings operation and
uses the same expected-byte/fail-closed principle. Canonical `emitter::emit`
continues to be deterministic semantic regeneration and does not claim exact
whole-file formatting or comment preservation.

## Mapped artifacts

Source mappings are keyed by public position: rule, condition, action, direct
action argument, and declaration. `Rule::actions` is a linear stream including
explicit control-flow lines, so these positions survive deterministic emission
and re-parsing. The decision record is
[ADR-0013](adr/0013-source-mapping-across-provider-boundary.md).

- **Shape guard.** Attached mappings record the program shape they were
  attached to: the rule count and each rule's condition and action counts. When
  the current shape differs, span accessors return `None` rather than a
  displaced span. Consumers that change program shape lose the affected
  mappings; they do not receive wrong locations.
- **Artifact formats.** `workshop-rs` defines the canonical Workshop artifact
  formats carried by provider protocols as opaque payloads:
  `workshop-rs/text-v1` is Workshop text; `workshop-rs/mapped-text-v1` is
  Workshop text plus a file table, the program shape, and position-keyed spans.
- **`SourceMap`.** `SourceMap` extracts the mapping from a span-bearing
  `Program` and applies it to a `Program` parsed from the same Workshop text.
  A shape mismatch rejects the whole mapping. Nodes without an authored origin
  carry no span and are reported as unmapped by consumers; source-language
  implementations own attribution for includes, macro expansion, and generated
  helpers.
- **Column units.** Columns follow `Position`: 1-based counts of Unicode scalar
  values. Producers convert into this unit; editor presentation converts to
  UTF-16.

## Shape guard

Attached mappings record the program shape they were attached to: the number
of rules, per-rule conditions and actions, and the declaration counts. The span
accessors return `None` once the public `Vec` fields no longer have that shape,
for example after inserting or removing a rule, condition, or action, instead
of returning a displaced span. Attaching a span again records the current shape
of the affected scope, so a consumer that changes the program shape re-attaches
the mapping it still wants, and one that builds a program incrementally attaches
after each addition; attaching `None` records the shape without a span.

## Canonical artifact formats

`workshop-rs` owns two canonical Workshop artifact formats. Provider protocols
carry them as opaque artifacts; they do not give the protocol Workshop
semantics.

- `workshop-rs/text-v1`: Workshop text alone.
- `workshop-rs/mapped-text-v1`: Workshop text plus a file table, the program
  shape, and position-keyed spans, encoded as one JSON document.

The public `SourceMap` extracts a mapping from a span-bearing `Program` and
applies it to a `Program` parsed from the same Workshop text. `MappedText`
pairs the text with its `SourceMap` and encodes the second format. Application
replaces the program's file table with the map's file table and its mapping with
the map's spans, so nodes without an entry carry no span and consumers report
evidence on them as unmapped. Mapped files are represented by path only, so
the program's retained source documents, including the Workshop text parsed
from the artifact, are dropped and `Program::source` returns `None` for them.
The whole mapping is rejected with a typed `SourceMapError`, leaving the
program unchanged, when the program shape differs from the recorded shape or
when any entry is invalid, duplicated, or an empty declaration entry.

A `mapped-text-v1` document has these members:

| Member | Content |
| --- | --- |
| `format` | `"workshop-rs/mapped-text-v1"` |
| `text` | The Workshop text |
| `files` | The file table: `{"path": ...}` entries; spans refer to entries by index |
| `shape` | `global_variables`, `player_variables`, `subroutines` counts and `rules`, a list of `{"conditions", "actions"}` counts |
| `spans` | Position-keyed entries, each tagged by `node` |

The `node` tags and their keys are `rule` (`rule`), `condition` (`rule`,
`condition`), `action` (`rule`, `action`), `action_argument` (`rule`, `action`,
`argument`), and `global_variable`, `player_variable`, and `subroutine`
(`index`). The first four carry a `span`; declarations carry `span`,
`name_span`, or both. A span is `{"file", "start", "end"}` with positions
`{"line", "column"}`. Only nodes with an authored origin have an entry, so
`spans` may be empty. Decoders ignore unknown members. The
mapping granularity is rule, condition, action, direct action argument, and
declarations; nested value mappings are not part of the format.

Columns follow `Position`: 1-based Unicode scalar values. Producers convert
from other units, and editor presentation converts to UTF-16.

## Mixed-source boundary

The raw Workshop parser accepts a complete supported Workshop document. It does
not interpret DEL/OSTW, OverPy, or other embedded source-language syntax. A
consumer embedding Workshop regions owns extraction and source-language
source attribution; it can attach the extracted Workshop source to a canonical program
file when available. Malformed or unsupported mixed content must be rejected or
reported incomplete by that consumer rather than being assigned guessed
Workshop semantics.
