# Release automation

`release-plz` maintains a Release PR from pushes to `main`. Merging that PR
is the normal release action. The merged Release PR runs the repository gates,
publishes `workshop-rs` before `workshop-rs-cli`, and creates the shared
`vX.Y.Z` tag. The repository workflow then ensures an unpublished draft GitHub
Release exists before calling the artifact workflow to build the CLI archives,
add checksums and catalog identity, and publish the completed GitHub Release.

## Repository configuration

Configure these repository resources before enabling the workflow:

1. Provide a repository or organization `GH_TOKEN` secret with access to this
   repository and permission to read/write contents and pull requests. The
   release workflow uses this token to create/update `release-plz-*` branches,
   Release PRs, the shared `vX.Y.Z` tag, and the draft GitHub Release when one
   was not created by release-plz itself.
2. Create an environment named `release` with required reviewers enabled.
   Store `CARGO_REGISTRY_TOKEN` in that environment and grant it permission to
   publish both crates. The environment is used only by the merged Release PR
   publication job.
3. Normal development remains PR-only; no direct `main` push exception is
   required. The dedicated `GH_TOKEN` is release-management infrastructure,
   not authority to bypass repository review policy.
4. Artifact publication remains a `workflow_call` from the release workflow.
   It does not depend on a tag event starting a second workflow. Release gates
   run after the Release PR is merged and before publication.

The release-plz job owns package publication and the canonical tag identity.
The repository workflow guarantees that the corresponding GitHub Release draft
exists before artifacts begin, and the called artifact workflow uses its scoped
`GITHUB_TOKEN` to attach assets and publish the completed release.

## Release identity and retries

Both packages use the `workshop-rs` release-plz `version_group`. Only the
library package owns the shared `vX.Y.Z` tag and GitHub Release; the CLI remains
published at the same version without a second public tag or release. Cargo's
dependency order makes the library publish before the CLI.

The release-plz release job is gated by format, clippy, tests, catalog check,
and `cargo package` for both crates. A failed publication or artifact run can
be retried: release-plz checks the registry and existing tag/release state, the
workflow reuses an existing unpublished draft when present, and the artifact
workflow uploads with `--clobber` before making the draft public. Do not
manually bump versions, create tags, or run `cargo publish` for a normal
release. Rerun the failed GitHub Actions run instead.

## Maintainer procedure

1. Merge normal changes through PRs using Conventional Commits.
2. Review the automatically maintained Release PR and its CI checks.
3. Merge the Release PR after the protected `release` environment is ready.
4. Approve the publication job when prompted. It publishes the two crates,
   establishes the tag and draft GitHub Release identity, then the called
   artifact workflow attaches the five platform archives, `SHA256SUMS.txt`,
   and `catalog-identity.json`, and publishes the release.

The resulting GitHub Release notes contain generated release notes, the exact
revision, and the CLI's machine-readable catalog version, digest, locale
coverage, and provenance identity.
