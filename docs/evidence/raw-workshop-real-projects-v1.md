# Raw Workshop real-project conformance rerun procedure

The machine-readable manifest in `raw-workshop-real-projects-v1.json` is the durable
corpus record for workshop-rs#47 and wright#189. It pins the source repository,
revision, path or generated-output procedure, artifact digest, locale, build
toolchain, reference compiler where applicable, and redistribution boundary.

The five pinned Workshop source inputs are committed under
`crates/workshop-rs/tests/fixtures/real-projects/`. A rerun uses those local
files and verifies their recorded SHA-256 before running the required harness.
The source revisions, acquisition procedures, and toolchains remain in the
JSON manifest so the fixtures can be regenerated or refreshed deliberately.

Acceptance results are not committed. The required CI job emits pass/fail,
residual classifications, and the harness log as CI output/artifacts. Local
diagnostic commands may be run against the fixture paths:

```text
workshop-rs-cli parse ARTIFACT --locale LOCALE
wright check --kind workshop --locale LOCALE ARTIFACT
wright lint --kind workshop --locale LOCALE ARTIFACT
```

The fixture must be treated as invalid input if its source revision, locale, or
digest differs from the manifest. Generated command output belongs in CI logs
or artifacts; it is not a replacement for the manifest or provenance record.
