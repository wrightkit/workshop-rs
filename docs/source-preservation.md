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

## Mixed-source boundary

The raw Workshop parser accepts a complete supported Workshop document. It does
not interpret DEL/OSTW, OverPy, or other embedded source-language syntax. A
consumer embedding Workshop regions owns extraction and source-language
source attribution; it can attach the extracted Workshop source to a canonical program
file when available. Malformed or unsupported mixed content must be rejected or
reported incomplete by that consumer rather than being assigned guessed
Workshop semantics.
