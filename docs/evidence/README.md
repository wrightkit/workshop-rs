# P0 corpus evidence

`raw-workshop-p0-v1.json` pins the five source repositories, revisions, raw
artifact SHA-256 values, and reacquisition commands. The raw files are not
redistributed. CI reacquires them into a private workspace and runs the
required five-project semantic gate:

```sh
WRIGHTKIT_P0_ARTIFACT_DIR=/path/to/reacquired-artifacts \
  cargo test -p workshop-rs --locked --test p0_corpus -- --ignored --nocapture
```

The test verifies each artifact digest, canonical validation, deterministic
emission, reparse, locale conversion, and semantic round-trip equivalence.
Residual classifications are emitted in CI logs/artifacts; they are not
duplicated as tracked acceptance-result files.
