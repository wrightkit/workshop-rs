# Corpus fixture provenance

These fixtures are raw Workshop texts used by the migrated en-US corpus
tests (parser, emitter, detect, roundtrip). They were extracted from the
Wright repository's compatibility corpus on 2026-08-16 during the canonical
core migration (Issue #2).

## Source and evidence

| Fixture | Extracted from | Original oracle identity |
| --- | --- | --- |
| `corpus/basic-rule.ws` | wright `compatibility/fixtures/synthetic/basic-rule/oracle.json` `compile.workshop` | OverPy 9.7.10, en-US emission |
| `corpus/control-flow.ws` | wright `compatibility/fixtures/synthetic/control-flow/oracle.json` `compile.workshop` | OverPy 9.7.10, en-US emission |
| `corpus/declarations-rules.ws` | wright `compatibility/fixtures/synthetic/declarations-rules/oracle.json` `compile.workshop` | OverPy 9.7.10, en-US emission |
| `corpus/expressions-values.ws` | wright `compatibility/fixtures/synthetic/expressions-values/oracle.json` `compile.workshop` | OverPy 9.7.10, en-US emission |
| `corpus/preprocessing.ws` | wright `compatibility/fixtures/synthetic/preprocessing/oracle.json` `compile.workshop` | OverPy 9.7.10, en-US emission |
| `corpus/receiver-calls.ws` | wright `compatibility/fixtures/synthetic/receiver-calls/oracle.json` `compile.workshop` | OverPy 9.7.10, en-US emission |
| `corpus/overpy-cake.ws` | wright `compatibility/fixtures/real-world/overpy-cake/oracle.json` `compile.workshop` | OverPy 9.7.10, en-US emission |
| `settings/pixelart.settings.ws` | wright `compatibility/fixtures/real-world/overpy-pixelart/oracle.json` `compile.workshop` settings section | OverPy 9.7.10, en-US emission |
| `settings/santa.settings.ws` | wright `compatibility/fixtures/real-world/overpy-santa/oracle.json` `compile.workshop` settings section | OverPy 9.7.10, en-US emission |
| `settings/pixelart.zh-CN.settings.ws` | deterministic conversion of `settings/pixelart.settings.ws` through the reviewed PR #9 locale corpus | reviewed `zh-CN` mappings, no fallback |

The original oracle snapshots record: OverPy version 9.7.10 (npm
`overpy@9.7.10`, git head `1e2688954302a402d076944b46db07efb14d7b61`,
GPL-3.0), language `en-US`.

## Extraction

Each `corpus/*.ws` file is the exact `compile.workshop` string of the source
`oracle.json`. Each `settings/*.settings.ws` file is the exact `settings { … }`
block (brace-balanced) of the source `compile.workshop` string. The SHA-256 of
every `corpus/*.ws` file equals the `workshopSha256` recorded in its source
`oracle.json` (extraction verified byte-identical).

## License and redistribution status

The Workshop spellings in these texts are Blizzard game content
(functional/interoperability data: action and value names as they appear in
the Workshop editor). The texts are reference-emission snapshots
(observed behavior of the pinned OverPy oracle), not OverPy source or
implementation internals. The workspace licensing policy
(workspace `AGENTS.md`, Wright `docs/licensing.md`, Wright ADR-0004) permits
observed reference behavior as an interoperability input with recorded
provenance; the catalog data itself is transcribed from the same evidence
class. Final redistribution review of the migrated corpus is tracked with the
first-release gate (see `docs/provenance.md`).

## Usage

Tests read these files via `env!("CARGO_MANIFEST_DIR")`; do not edit them
incidentally. A corpus change is a reviewed, evidenced change like any
catalog data change and must be recorded here.
