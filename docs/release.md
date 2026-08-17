# Release automation

`Release` is a manually triggered GitHub Actions workflow. Dispatch it from
the repository's `main` branch and select `patch`, `minor`, or `major`; the
normal patch path requires no version text entry.

## Repository configuration

The repository needs:

1. An environment named `release` with required reviewers enabled for the
   publication jobs.
2. An environment secret named `CARGO_REGISTRY_TOKEN`. The token must be
   allowed to publish both `workshop-rs` and `workshop-rs-cli`; it is never
   printed by the workflow.
3. A ruleset/environment exception allowing the release workflow's
   `github-actions[bot]` to push the deterministic version commit to `main`
   and the immutable `vX.Y.Z` tag. Normal development remains PR-only.

The workflow grants `contents: read` by default and `contents: write` only to
the prepare and GitHub Release jobs. Registry publication is gated by the
protected `release` environment. A future crates.io trusted-publishing
configuration may replace the registry token, but it must preserve the same
environment approval and package-order guarantees.

## Release identity and retries

The workspace version, both package versions, Cargo.lock, the release commit,
the `vX.Y.Z` tag, registry packages, and GitHub Release all refer to one
revision. The library is published before the CLI because the CLI declares a
matching registry-compatible `workshop-rs` dependency while retaining its
local path for development.

The workflow detects an incomplete version/tag/release and resumes it. It
skips crates already visible at the target version, reuses an existing
immutable tag, and resumes a draft GitHub Release. Once a tag has a published
GitHub Release and both crates are present, a new dispatch computes the next
selected bump instead of reusing the completed version.

Platform artifacts are built for Linux x86_64/aarch64, macOS x86_64/aarch64,
and Windows x86_64. The final GitHub Release contains the five archives,
`SHA256SUMS.txt`, generated notes, the exact revision, and the CLI's
machine-readable catalog identity (catalog version, digest, and locale
coverage).
