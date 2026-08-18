# ADR-0005: Seasonal Workshop client validation workflow

## Status

Accepted as the repository-owned offline contract for #21. Live-client
execution remains a manual maintainer boundary.

## Decision

`workshop_rs::live_capture::LiveCapture` is the machine-readable envelope for a
capture made by a maintainer from the #19 census. It pins the capture ID,
game/client/season metadata, capture time and environment, client locale,
catalog identity, census schema/digest/shards, raw exported Workshop artifact
provenance, and feature-attributed #18 results. Validation requires every
result to be `live-client` evidence with matching catalog, locale, raw
artifact, game, and capture-time provenance. The bundled runtime remains
offline; no network catalog or client integration is introduced.

`workshop-rs-cli seasonal-diff <previous.json> <current.json> [--json]`
validates both documents and emits a deterministic structured diff. Changes
are classified as `locale`, `catalog`, `content`, `semantic-schema`, or
`runtime-uncertainty`; feature identities and case IDs are retained on
feature-level entries. A runtime classification is deliberately not a claim
of gameplay behavior: import/export acceptance cannot establish runtime
correctness.

No real client capture is committed by this issue. Unit tests use explicitly
labelled synthetic schema/diff inputs and are not evidence of an Overwatch
client observation.

## Maintainer procedure

1. Run `workshop-rs-cli census --json` from the reviewed #19 head and retain
   the exported shard definition, catalog identity, census digest, and case
   to #18 feature mapping.
2. Assemble the required shard probes into an importable Workshop text
   document. Keep the generated probe separate from the client-exported raw
   artifact.
3. In the current Overwatch client, import the en-US probe, record the
   observable client version/season and capture time, then export or copy the
   resulting Workshop text without editing it. Hash and preserve the raw
   artifact outside the repository, and record its immutable path/revision,
   license or retention note, and SHA-256 in `rawArtifact`.
4. Repeat the same probe in zh-CN. The locale must be recorded on the capture
   and on every #18 result; do not infer a locale from the text after capture.
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

This workflow proves only what the recorded client import/export and metadata
support. It does not automate startup, login, locale switching, or gameplay,
and it does not claim exhaustive runtime correctness. If no Overwatch client
is available, the validated offline schema and diff contracts are the complete
result and the manual capture step remains explicitly open.
