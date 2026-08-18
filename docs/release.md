# Release automation

`release-plz` maintains a Release PR from pushes to `main`. Merging that PR
is the normal release action. The merged Release PR runs the repository gates,
publishes `workshop-rs` before `workshop-rs-cli`, and creates one draft
`vX.Y.Z` tag/release. The tag workflow builds the CLI archives, adds checksums
and catalog identity, then publishes the draft GitHub Release.

## Repository configuration

Configure these repository resources before enabling the workflow:

1. Create a fine-grained `RELEASE_PLZ_TOKEN` with repository access and
   `Contents: read and write` plus `Pull requests: read and write`. Use it for
   checkout and release-plz. The default `GITHUB_TOKEN` cannot trigger the tag
   workflow or CI for a Release PR.
2. Create an environment named `release` with required reviewers enabled.
   Store `CARGO_REGISTRY_TOKEN` in that environment and grant it permission to
   publish both crates. The environment is used only by the merged Release PR
   publication job.
3. Allow the release-plz token to create `release-plz-*` branches, update
   Release PRs, create immutable `vX.Y.Z` tags, and create draft releases.
   Normal development remains PR-only; no direct `main` push exception is
   required.
4. Keep the normal CI checks required on Release PRs. The PAT is deliberate so
   those checks run for the bot-created PR and the tag event starts the
   artifact workflow.

The release workflow grants only the permissions needed by each job. The
artifact workflow uses the run's `GITHUB_TOKEN` only to update the draft
release after the tag has triggered it.

## Release identity and retries

Both packages use the `workshop-rs` release-plz `version_group`. Only the
library package owns the shared `vX.Y.Z` tag and GitHub Release; the CLI remains
published at the same version without a second public tag or release. Cargo's
dependency order makes the library publish before the CLI.

The release-plz release job is gated by format, clippy, tests, catalog check,
and `cargo package` for both crates. A failed publication or artifact run can
be retried: release-plz checks the registry and existing tag/release state, and
the artifact workflow uploads with `--clobber` before making the draft public.
Do not manually bump versions, create tags, or run `cargo publish` for a normal
release. Rerun the failed GitHub Actions run instead.

## Maintainer procedure

1. Merge normal changes through PRs using Conventional Commits.
2. Review the automatically maintained Release PR and its CI checks.
3. Merge the Release PR after the protected `release` environment is ready.
4. Approve the publication job when prompted. It publishes the two crates and
   creates the draft release; the tag workflow then attaches the five platform
   archives, `SHA256SUMS.txt`, and `catalog-identity.json`, and publishes the
   release.

The resulting GitHub Release notes contain the generated release-plz notes,
the exact revision, and the CLI's machine-readable catalog version, digest,
locale coverage, and provenance identity.
