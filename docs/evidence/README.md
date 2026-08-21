# P0 corpus evidence

`raw-workshop-p0-v1.json` pins the five source repositories, revisions, raw
artifact SHA-256 values, and vendored fixture paths. CI runs the required
five-project semantic gate directly against those local source inputs:

```sh
cargo test -p workshop-rs --locked --test p0_corpus -- --nocapture
```

The test verifies each artifact digest, canonical validation, deterministic
emission, reparse, locale conversion, and semantic round-trip equivalence.
Residual classifications are emitted in CI logs/artifacts; they are not
duplicated as tracked acceptance-result files.
