# ADR-0003: Canonical sharded Workshop feature census

## Status

Accepted

## Decision

The CLI census runner derives deterministic shards from the canonical catalog,
the reviewed settings table, and Workshop IR capabilities owned by
`workshop-rs`. It does not import OPY/DEL inventories or claim live
client/runtime behavior.

Each `CensusCase` has a stable case ID, one or more `FeatureId` values,
canonical en-US source text, and an explicit support classification. Catalog
entries and enum members are emitted from the loaded canonical catalog;
content IDs use their domain-qualified canonical enum-member identities.
Settings, variables, subroutines, control flow, strings, and localization
cases use `workshop-rs`-owned namespaces.

`Census::run` executes each case independently through parse, WIR validation,
canonical identity validation, emission, semantic round-trip, and en-US/zh-CN
conversion checks. The WIR shard is derived from the WIR-owned
`CENSUS_CAPABILITIES` registry. A case is matched only when its conversion is
also equal to an independently recorded reference source; generated probes
without such an artifact remain inconclusive even when all offline gates pass.
The report retains every `ConformanceResult`, including unsupported,
known-gap, unexpected-regression, and inconclusive states.
`Census::identity` exposes a deterministic SHA-256 digest of the reviewed,
serialized shard definition together with the sorted shard IDs. Every
`CensusReport` carries that authoritative `CensusIdentity`; report validation
checks its schema, digest shape, and shard list. `CensusReport::validate_against`
also binds catalog feature IDs and result case IDs to the actual catalog and
declared shards. `Census::export_json` exports shard definitions without
making the export itself an oracle.

The census localization inputs are probes, not independent locale evidence.
A census result can be classified as `matched` only when an independent
expectation source is recorded through the conformance contract.

The report's comparison fields describe the independent semantic/normalized
gate; they do not turn the implementation's own output into an expected
oracle. Focused parser/catalog/emitter tests remain complementary and are not
replaced by census totals.

## Consequences

The census runner is tooling owned by `workshop-rs-cli`
(`crates/workshop-rs-cli/src/census.rs`). Its probes exercise the semantic
crate and its contract tests; they are not part of the semantic crate's source
domain taxonomy.

Live-client workflows may assemble the same shards into probes while retaining
feature attribution. A census result represents offline evidence; live-client
or other independent claims require the corresponding expectation source and
evidence provenance.
