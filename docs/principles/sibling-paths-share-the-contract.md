# sibling-paths-share-the-contract

**Applies when:** a second path faces a contract an existing path already enforces: a tolerant carry beside strict parsing, `source_edit` beside `write`, `check` beside `emit`, CLI beside library, a scoped query beside an unscoped one.

**Principle:** Reuse the established path's check, or enumerate its obligations before writing the new one. A new path must not accept what its sibling rejects, nor produce what its sibling cannot consume.

**Changes the choice between:** writing a fresh, locally sufficient validation in the new path, and routing the new path through the sibling's check.

**Enforced by:** judgment at authoring time. Regression tests that assert sibling paths agree on shared inputs catch drift afterward; they do not choose the shared check.

**Why:** every pairing drifted the same way, in signature rendering, scoped lookup, verbatim carriage, `source_edit`, `check`, and the CLI, and each fix converged on sharing the sibling's check.

**Detail:** promoted from the former `LESSONS.md` entry; incidents are in the delivering PR.
