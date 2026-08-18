# Release automation

`release-plz` maintains a Release PR from pushes to `main`. Merging that PR
is the normal release action. The merged Release PR runs the repository gates,
publishes `workshop-rs` before `workshop-rs-cli`, and creates one draft
`vX.Y.Z` tag/release. The same workflow then calls the artifact workflow to
build the CLI archives, add checksums and catalog identity, and publish the
draft GitHub Release.

## Repository configuration

Configure these repository resources before enabling the workflow:

1. Create an environment named `release` with required reviewers enabled.
   Store `CARGO_REGISTRY_TOKEN` in that environment and grant it permission to
   publish both crates. The environment is used only by the merged Release PR
   publication job.
2. Allow the repository `GITHUB_TOKEN` to create `release-plz-*` branches,
   update Release PRs, create immutable `vX.Y.Z` tags, and create draft
   releases. Normal development remains PR-only; no direct `main` push
   exception is required.
3. The default token does not start a separate workflow from a tag event, so
   artifacts are invoked through `workflow_call` in the same release run.
   Release gates run after the Release PR is merged and before publication;
   CI checks on the bot-created Release PR are not relied on as the release
   gate.

The release workflow grants only the permissions needed by each job. The
artifact workflow uses the caller's `GITHUB_TOKEN` to update the draft release
after the release-plz job returns its tag output.

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
   creates the draft release; the called artifact workflow then attaches the
   five platform archives, `SHA256SUMS.txt`, and `catalog-identity.json`, and
   publishes the release.

The resulting GitHub Release notes contain the generated release-plz notes,
the exact revision, and the CLI's machine-readable catalog version, digest,
locale coverage, and provenance identity.
