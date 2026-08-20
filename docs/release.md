# Release automation

`workshop-rs` uses two upstream release tools with separate responsibilities:

- `release-plz` maintains the Release PR, publishes the Rust crates to crates.io,
  and creates the canonical `vX.Y.Z` tag.
- `dist` owns binary distribution from that tag: release planning, five-platform
  CLI builds, per-artifact SHA-256 files, `catalog-identity.json`, and the final
  GitHub Release.

The repository does not maintain a second tag/Release state machine around
those tools.

## Repository configuration

1. Configure `GH_TOKEN` as a repository or organization secret with permission
   to update this repository and maintain pull requests. `release-plz` uses it
   for the Release PR and canonical tag. A dedicated token is retained here so
   the tag created by release automation can trigger the tag-based `dist`
   workflow.
2. Keep `CARGO_REGISTRY_TOKEN` in the protected `release` environment with
   permission to publish both `workshop-rs` and `workshop-rs-cli`. This PR keeps
   the already-proven registry credential path while the release orchestration
   is simplified. crates.io Trusted Publishing can replace this token in a
   separate change after both crates have been configured and verified there.
3. Keep normal CI required on the Release PR. Release-specific orchestration
   should not duplicate the repository's normal Rust test suite.

## Release flow

Normal pushes to `main` run the two standard `release-plz` jobs:

1. `release-plz release` publishes any workspace versions that are present in
   Git but not yet in crates.io. The library and CLI share the `workshop-rs`
   version group; the library is published before the CLI. The `workshop-rs`
   package owns the single public `vX.Y.Z` tag. `release-plz` does not create a
   GitHub Release.
2. `release-plz release-pr` creates or refreshes the next Release PR.

When the canonical tag reaches GitHub, the dist-generated `Release` workflow
runs:

```text
vX.Y.Z
  -> dist plan
  -> build workshop-rs-cli for five targets
  -> build global artifacts/checksums
  -> create the GitHub Release with the complete artifact set
  -> announce
```

The supported targets are:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

Linux ARM64 uses GitHub's native `ubuntu-24.04-arm` runner rather than a
repository-maintained cross-linker setup.

`dist` preserves the existing `.tar.gz`/`.zip` archive formats and emits a
`.sha256` file for each archive. `catalog-identity.json` is configured as a
`dist` extra artifact and is generated from `workshop-rs-cli version --json`.
The previous aggregate `SHA256SUMS.txt` file is replaced by dist's standard
per-artifact checksum files.

## Pull-request validation

The dist workflow runs `dist plan` on pull requests. This validates release tag
interpretation, the selected package, target matrix, and artifact plan without
performing publication side effects. The generated workflow should be updated
through `dist init`/`dist generate` when the pinned dist version or distribution
configuration changes; do not hand-maintain a parallel release implementation.

## Failure and recovery

The release systems are intentionally not treated as one atomic transaction.
crates.io, Git tags, and GitHub Releases are separate external states.

- If crates.io publication fails, rerun the failed `release-plz` job; already
  published crate versions are skipped.
- If crate publication succeeds but tag creation fails, repair/retry the
  canonical tag before invoking distribution. Do not create a replacement
  version solely to repair binary distribution.
- If dist fails before hosting, rerun the tag workflow after fixing the actual
  build/configuration problem. GitHub Release creation remains owned by dist.
- Do not recreate repository-specific draft/published Release state machines to
  automate rare recovery cases. Prefer explicit maintainer recovery when an
  external service is left in an unusual partial state.

## Maintainer procedure

1. Merge normal changes through PRs using Conventional Commits.
2. Review the automatically maintained Release PR and its normal CI checks.
3. Merge the Release PR when the version is ready.
4. Approve the protected `release` environment if required.
5. `release-plz` publishes the crates and creates `vX.Y.Z`.
6. The tag-triggered dist workflow builds the CLI artifacts and publishes the
   GitHub Release.

For routine releases, maintainers should not manually bump versions, run
`cargo publish`, create GitHub Releases, or edit generated dist CI.
