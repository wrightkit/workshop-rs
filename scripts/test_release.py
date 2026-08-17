import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from release import Version, bump_manifests, plan_release


class ReleasePlanningTests(unittest.TestCase):
    def test_first_release_uses_selected_bump(self):
        plan = plan_release(Version.parse("0.1.0"), [], "patch")
        self.assertEqual((str(plan.version), plan.mode, plan.ref), ("0.1.1", "new", "HEAD"))

    def test_unfinished_tag_resumes_without_bumping(self):
        plan = plan_release(Version.parse("0.1.1"), ["v0.1.1"], "minor")
        self.assertEqual((str(plan.version), plan.mode, plan.ref), ("0.1.1", "resume", "v0.1.1"))

    def test_completed_tag_starts_the_next_release(self):
        plan = plan_release(Version.parse("0.1.1"), ["v0.1.1"], "minor", release_complete=True)
        self.assertEqual((str(plan.version), plan.mode, plan.ref), ("0.2.0", "new", "HEAD"))

    def test_unpublished_version_resumes_the_existing_release_commit(self):
        plan = plan_release(
            Version.parse("0.1.2"), ["v0.1.1"], "patch", release_commit="abc123"
        )
        self.assertEqual((str(plan.version), plan.mode, plan.ref, plan.commit), ("0.1.2", "resume", "abc123", "abc123"))

    def test_bumps_reset_lower_components(self):
        self.assertEqual(str(Version.parse("1.2.3").bump("minor")), "1.3.0")
        self.assertEqual(str(Version.parse("1.2.3").bump("major")), "2.0.0")

    def test_bump_updates_workspace_and_publishable_cli_dependency(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "crates/workshop-rs-cli"
            cli.mkdir(parents=True)
            (root / "Cargo.toml").write_text('[workspace.package]\nversion = "0.1.0"\n')
            (cli / "Cargo.toml").write_text(
                '[dependencies]\nworkshop-rs = { path = "../workshop-rs", version = "0.1.0" }\n'
            )
            bump_manifests(Version.parse("0.2.0"), root)
            self.assertIn('version = "0.2.0"', (root / "Cargo.toml").read_text())
            self.assertIn('version = "0.2.0"', (cli / "Cargo.toml").read_text())


if __name__ == "__main__":
    unittest.main()
