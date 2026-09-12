# ADR-0005: Seasonal Workshop client validation workflow

## Status

Accepted

## Decision

`workshop-rs-cli` provides the machine-readable `LiveCapture` envelope for a
capture derived from a census artifact. It pins the capture ID,
game/client/season metadata, capture time and environment, client locale,
catalog identity, census schema/digest/shards, raw exported Workshop artifact
provenance, and feature-attributed conformance results. Validation requires every
result to be `live-client` evidence with matching catalog, locale, raw
artifact, game, and capture-time provenance. The bundled runtime is offline; no
network catalog or client integration is introduced.

`workshop-rs-cli seasonal-diff <previous.json> <current.json> [--json]`
validates both documents and emits a deterministic structured diff. Changes
are classified as `locale`, `catalog`, `content`, `semantic-schema`, or
`runtime-uncertainty`; feature identities and case IDs are retained on
feature-level entries. A runtime classification is deliberately not a claim
of gameplay behavior: import/export acceptance cannot establish runtime
correctness.

Synthetic schema and diff inputs exercise the validation contract; they are not
evidence of an Overwatch client observation.

## Maintainer procedure

1. Run `workshop-rs-cli census --json` from a reviewed census input and retain
   the exported shard definition, catalog identity, census digest, and case
   to feature mapping.
2. Assemble the required shard probes into an importable Workshop text
   document. Keep the generated probe separate from the client-exported raw
   artifact.
3. In an Overwatch client, import the en-US probe, record the
   observable client version/season and capture time, then export or copy the
   resulting Workshop text without editing it. Hash and preserve the raw
   artifact outside the repository, and record its immutable path/revision,
   license or retention note, and SHA-256 in `rawArtifact`.
4. Repeat the same probe in zh-CN. The locale must be recorded on the capture
   and on every conformance result; do not infer a locale from the text after
   capture.
5. Assemble the manually recorded results into a `LiveCapture` document and
   run `seasonal-diff` against the prior capture. Keep both JSON reports and
   both raw artifacts; do not replace the prior expectation with the new
   observation.
6. Review every classified change. Locale-only changes go to localization
   evidence review; catalog/content or semantic/schema changes become focused
   catalog/semantic issues with their raw artifacts and feature IDs attached;
   runtime-only uncertainty requires a separate reproducible gameplay
   experiment. A new capture is never accepted as canonical by automation.

## Evidence boundary

The capture schema and diff command are implemented by `workshop-rs-cli`
(`crates/workshop-rs-cli/src/live_capture.rs`). The semantic crate supplies
the Workshop contracts they validate but does not own a capture implementation
domain.

This workflow proves only what the recorded client import/export and metadata
support. It does not automate startup, login, locale switching, or gameplay,
and it does not claim exhaustive runtime correctness. Offline schema and diff
validation does not establish live-client claims; those claims require a
recorded capture with the provenance described above.
