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
| `corpus/overpy-cake.ws` | migrated in workshop-rs `44c04a1166f3df9a495fc04c79e0ae2adc4542d2` from wright `compatibility/fixtures/real-world/overpy-cake/oracle.json` `compile.workshop` | OverPy 9.7.10, en-US emission |
| `corpus/minimized/overpy-cake-loop.ws` | minimized from `corpus/overpy-cake.ws`, retaining its variable/loop/array/random-value interaction | workshop-rs migration `44c04a1166f3df9a495fc04c79e0ae2adc4542d2`, linked to the pinned wright OverPy 9.7.10 oracle |
| `settings/pixelart.settings.ws` | wright `compatibility/fixtures/real-world/overpy-pixelart/oracle.json` `compile.workshop` settings section | OverPy 9.7.10, en-US emission |
| `settings/santa.settings.ws` | wright `compatibility/fixtures/real-world/overpy-santa/oracle.json` `compile.workshop` settings section | OverPy 9.7.10, en-US emission |
| `settings/pixelart.zh-CN.settings.ws` | deterministic conversion of `settings/pixelart.settings.ws` through the reviewed PR #9 locale corpus | reviewed `zh-CN` mappings, no fallback |

The original wright oracle snapshot is pinned in the executable manifest by
its wright revision and SHA-256. It records OverPy 9.7.10, the original
OverPy source revision, and language `en-US`; the oracle JSON is not
redistributed here.

## Extraction

Each `corpus/*.ws` file is the exact `compile.workshop` string of the source
`oracle.json`. Each `settings/*.settings.ws` file is the exact `settings { … }`
block (brace-balanced) of the source `compile.workshop` string. The SHA-256 of
every `corpus/*.ws` file equals the `workshopSha256` recorded in its source
`oracle.json` (extraction verified byte-identical).

## Provenance status

The Workshop spellings in these texts are reference-emission inputs. The
catalog data is workshop-rs-owned mapping data transcribed from recorded
evidence; the external JSON evidence artifact is not redistributed.

## Usage

Tests read these files via `env!("CARGO_MANIFEST_DIR")`; do not edit them
incidentally. A corpus change is a reviewed, evidenced change like any
catalog data change and must be recorded here.

## Real-project admission

`corpus/real-projects.json` is the executable #20 manifest. It keeps the
complete `overpy-cake.ws` project-level case and the minimized loop case as
separate complementary layers. Each source tuple points to the immutable
workshop-rs migration artifact, while its expectation tuple points to the
immutable wright oracle artifact. The original external oracle JSON is not
redistributed. The runner validates fixture digests from local files and the
required pinned oracle digest fields; it never recomputes a historical
expectation from current content.
The offline runner records parse/WIR behavior as #18 conformance results and
never replaces the pinned expectation with current implementation output.
