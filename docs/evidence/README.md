# P0 corpus evidence

`raw-workshop-p0-v1.json` pins the five source repositories, revisions, raw
artifact SHA-256 values, and reacquisition commands. The raw files are not
redistributed. Reacquire them into a private directory using the manifest,
then run the durable semantic census:

```sh
WRIGHTKIT_P0_ARTIFACT_DIR=/path/to/reacquired-artifacts \
  cargo test -p workshop-rs --locked --test p0_corpus -- --ignored --nocapture
```

The test verifies each artifact digest and the expected semantic issue count.
`workshop-p0-v1.json` records the resulting grouped classifications; remaining
project-defined and evidence-insufficient constructs stay visible rather than
being promoted to canonical identities.
