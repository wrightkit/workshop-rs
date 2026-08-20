# Raw Workshop P0 rerun procedure

The machine-readable manifest in `raw-workshop-p0-v1.json` is the durable
corpus record for workshop-rs#47 and wright#189. It pins the source repository,
revision, path or generated-output procedure, artifact digest, locale, build
toolchain, reference compiler where applicable, and redistribution boundary.

The full five artifacts are intentionally not committed: three repositories
assert no redistribution license, and the generated outputs require maintainer
review. A rerun must acquire each artifact from the pinned source revision or
URL, verify the recorded SHA-256, and use the local artifact path for both
commands:

```text
workshop-rs-cli parse ARTIFACT --locale LOCALE
wright check --kind workshop --locale LOCALE ARTIFACT
wright lint --kind workshop --locale LOCALE ARTIFACT
```

The artifact must be treated as invalid evidence if acquisition, source
revision, locale, or digest differs from the manifest. Command output belongs
in a separately named evidence directory; it is not a replacement for the
manifest or provenance record.
