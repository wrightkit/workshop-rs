# ADR-0012: Tests-first Workshop verification

- Status: Accepted
- Date: 2026-09-20
- Related: workshop-rs #241; wrightkit/.github #60

## Context

Workshop correctness is protected by tests and the test data they consume.
The previous conformance contract added a parallel vocabulary for classifying
test inputs, expectation sources, implementation observations, and client
captures. Those distinctions duplicated ordinary test concerns and preserved
point-in-time task metadata in current manifests.

The repository still needs feature-attributed results, expected and observed
artifacts, locale and catalog identities, immutable source provenance, and
client/runtime metadata. It does not need a generic verification architecture
to retain them.

## Decision

1. Unit, integration, regression, compatibility, corpus, census, and client
   workflows are tests or test-data workflows. Compatibility and client
   observations are behavior contracts tested at their respective boundaries.
2. A machine-readable result records only the direct test concerns it needs:
   case and feature identities, status, expected/observed comparison, test
   input identity, catalog identity, locale, and a structured reason for
   non-matches.
3. TestArtifact retains repository/source identity, immutable revision/path,
   digest, and license when those facts are required for attribution,
   licensing, or reproducibility. It is not a classification of trust or an
   expectation basis.
4. Client capture documents retain game, client, season, timestamp,
   environment, locale, catalog, census, raw exported input, and the test
   results produced from that capture. Offline capture validation does not
   claim gameplay/runtime correctness.
5. Current manifests and fixture metadata do not carry Issue/PR/task tracking,
   derived-case bookkeeping, historical corrections, or run observations
   when Git, Issues, PRs, and CI already preserve that history.
6. Expected results remain independent test data or reference comparisons.
   Implementation output is never promoted to an expected result merely
   because a local test passes.
7. A pinned source or oracle identity without materialized expected output is
   not a comparison: the runner reports `inconclusive` with `NotComparable`,
   reporting `matched` only after an expected-versus-observed comparison
   actually runs.

## Consequences

workshop-rs-cli keeps the smallest shared result shape needed by census,
corpus, and client-capture tooling. Feature attribution and explicit
non-matching statuses remain observable, while contributors can understand the
verification surface through ordinary tests, fixtures, references, and source
provenance. Real-project manifests that pin an oracle identity without
redistributing its expected output remain useful parser/WIR tests, but their
offline result is explicitly inconclusive rather than a fabricated match.

ADRs 0002 through 0005 remain historical records of the superseded model.
Their surviving fixture, census, and client-workflow decisions are routed
through this ADR and the current source/tests rather than through their former
verification taxonomy.
