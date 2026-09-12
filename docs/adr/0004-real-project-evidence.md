# ADR-0004: Provenance-linked real-project and minimized evidence

## Status

Accepted

## Decision

The evidence manifest can retain a complete real-project fixture and a separate
minimized case derived from it. Each case records its own immutable
`repository/revision/path` tuple and pinned SHA-256. An expectation may point to
an immutable external oracle artifact without redistributing that artifact.

The initial evidence recorded for this decision used the `overpy-cake.ws`
project and a minimized loop case derived from it; the Workshop fixture and the
external OverPy 9.7.10 oracle were identified by immutable provenance.

`workshop-rs-cli corpus <manifest> [--json]` runs the manifest offline. It
reads the preserved Workshop source, parses and validates it through the
canonical catalog/WIR path, and emits structured conformance results. A parse or WIR
failure becomes an unexpected regression unless an explicit known-gap record
matches the observed diagnostic. Known gaps and unsupported states remain
visible and do not count as matched. The runner never creates an expectation
from the implementation output.

Full-project and minimized cases are complementary: the minimized case makes
the feature-level regression easy to diagnose, while the complete project
preserves cross-rule and integration interactions.

## Admission rules

Each case must identify its source repository, immutable revision, source path,
pinned digest, license, expectation basis, and canonical feature IDs. A minimized case
must retain a `derivedFrom` link to its complete project case. Any changed
expected status or evidence source requires independent review; a green local
run alone is not grounds to rewrite an expectation.
