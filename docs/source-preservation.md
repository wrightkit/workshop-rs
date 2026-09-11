# Workshop source preservation

`workshop-rs` keeps canonical Workshop meaning in `wir::Program`. Authored
source is an optional capability on the existing source-file registry, so
programmatically constructed WIR does not need synthetic source metadata.

## Supported source

`parser::parse` and `parser::parse_with_context` retain the exact input in a
`SourceDocument` attached to file 0. A consumer constructing a program can use
`SourceFile::with_source` or `SourceFile::set_source` when it has authored
source for a file. The document preserves every byte,
including whitespace and newlines, and indexes `//` line comments outside
string literals. Other trivia is intentionally not assigned semantic identity;
it remains available in the exact source text.

Canonical nodes keep their existing optional `Span` values. A comment is
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

The settings API's `SettingSourceEdit` remains the typed settings operation and
uses the same expected-byte/fail-closed principle. Canonical `emitter::emit`
continues to be deterministic semantic regeneration and does not claim exact
whole-file formatting or comment preservation.

## Mixed-source boundary

The raw Workshop parser accepts a complete supported Workshop document. It does
not interpret DEL/OSTW, OverPy, or other embedded source-language syntax. A
consumer embedding Workshop regions owns extraction and source-language
provenance; it can attach the extracted Workshop source to a canonical program
file when available. Malformed or unsupported mixed content must be rejected or
reported incomplete by that consumer rather than being assigned guessed
Workshop semantics.
