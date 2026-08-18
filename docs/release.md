# Release automation

`release-plz` maintains a Release PR from pushes to `main`. Merging that PR
is the normal release action. The merged Release PR runs the repository gates
and publishes `workshop-rs` before `workshop-rs-cli`. Repository-owned workflow
steps then establish the canonical `vX.Y.Z` tag and draft GitHub Release before
the artifact workflow builds the CLI archives, adds checksums and catalog
identity, and publishes the completed GitHub Release.

## Repository configuration

Configure these repository resources before enabling the workflow:

1. Provide a repository or organization `GH_TOKEN` secret with access to this
   repository and permission to read/write contents and pull requests. The
   release workflow uses this token for `release-plz-*` branches and Release
   PRs, and for repository-owned tag and draft Release management.
2. Create an environment named `release` with required reviewers enabled.
   Store `CARGO_REGISTRY_TOKEN` in that environment and grant it permission to
   publish both crates. The environment is used only by the merged Release PR
   publication job.
3. Normal development remains PR-only; no direct `main` push exception is
   required. Release-management credentials do not bypass repository review
   policy.
4. Artifact publication remains a `workflow_call` from the release workflow.
   It does not depend on a tag event starting a second workflow.

`release-plz` owns version calculation, Release PR maintenance, and crates.io
publication. It is deliberately configured not to create Git tags or GitHub
Releases. After publication succeeds, the repository workflow derives the
release version from the merged Cargo metadata, creates or validates the
canonical tag against the merge commit, and creates or reuses the draft GitHub
Release. The called artifact workflow uses its scoped `GITHUB_TOKEN` to attach
assets and publish the completed release.

## Release identity and retries

Both packages use the `workshop-rs` release-plz `version_group` and therefore
must have the same release version. There is one public `vX.Y.Z` tag and one
GitHub Release for the repository; the CLI does not get a second public tag or
release. Cargo's dependency order makes the library publish before the CLI.

The release-plz release job is gated by format, clippy, tests, catalog check,
and `cargo package` for both crates. The workflow is recoverable across partial
publication: release-plz skips crate versions already present in the registry,
the repository workflow independently resolves the intended release identity,
validates or creates the tag, and creates or reuses an unpublished draft. The
artifact workflow uploads with `--clobber` before publishing the draft.

If a GitHub Release is already public, the workflow does not move it back to
draft. A complete published release is treated as already finished; an
incomplete published release fails explicitly for maintainer recovery. This
keeps the normal path compatible with immutable-release semantics.

Do not manually bump versions or run `cargo publish` for a normal release.
Retry the failed GitHub Actions job so the workflow can resume from the
externally visible registry/tag/draft state.

## Maintainer procedure

1. Merge normal changes through PRs using Conventional Commits.
2. Review the automatically maintained Release PR and its CI checks.
3. Merge the Release PR after the protected `release` environment is ready.
4. Approve the publication job when prompted. It publishes both crates,
   establishes the canonical tag and draft GitHub Release, then invokes the
   artifact workflow.
5. The artifact workflow attaches the five platform archives,
   `SHA256SUMS.txt`, and `catalog-identity.json`, then publishes the completed
   GitHub Release.

The resulting GitHub Release notes contain generated release notes, the exact
revision, and the CLI's machine-readable catalog version, digest, locale
coverage, and provenance identity.
