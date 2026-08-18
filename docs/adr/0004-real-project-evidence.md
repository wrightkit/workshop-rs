# ADR-0004: Provenance-linked real-project and minimized evidence

## Status

Accepted for the offline real-project evidence lane of #10.

## Decision

`tests/fixtures/corpus/real-projects.json` is the reviewed manifest for the
first preserved real-project evidence case. It retains the complete
`overpy-cake.ws` project-level fixture and a separate minimized loop fixture
derived from it. The manifest records the immutable repository commit, source
path, license, fixture class, canonical #18 feature IDs, and the pinned
OverPy 9.7.10 en-US oracle provenance. The external oracle artifact is
referenced but not redistributed.

`workshop-rs-cli corpus <manifest> [--json]` runs the manifest offline. It
reads the preserved Workshop source, parses and validates it through the
canonical catalog/WIR path, and emits structured #18 results. A parse or WIR
failure becomes an unexpected regression unless an explicit known-gap record
matches the observed diagnostic. Known gaps and unsupported states remain
visible and do not count as matched. The runner never creates an expectation
from the current implementation output.

Full-project and minimized cases are complementary: the minimized case makes
the feature-level regression easy to diagnose, while the complete project
preserves cross-rule and integration interactions.

## Admission rules

Future cases must identify their source repository, immutable revision, source
path, license, expectation basis, and canonical feature IDs. A minimized case
must retain a `derivedFrom` link to its complete project case. Any changed
expected status or evidence source requires independent review; a green local
run alone is not grounds to rewrite an expectation.
