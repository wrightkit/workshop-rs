# Real-project evidence

`raw-workshop-p0-v1.json` pins the five source repositories, revisions, raw
artifact SHA-256 values, and vendored fixture paths. The evidence is exercised
by the parser, emitter, and locale scenario tests against those local source
inputs:

```sh
cargo test -p workshop-rs --locked --test parser pinned_real_projects -- --nocapture
cargo test -p workshop-rs --locked --test emitter pinned_real_projects -- --nocapture
cargo test -p workshop-rs --locked --test locale pinned_real_projects -- --nocapture
```

Together these tests verify each artifact digest, semantic residual policy,
canonical validation, deterministic emission, reparse, locale conversion, and
semantic round-trip equivalence. Residual classifications are emitted in CI
logs/artifacts; they are not duplicated as tracked acceptance-result files.
