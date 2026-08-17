#!/usr/bin/env python3
"""Deterministic version planning and manifest mutation for the release workflow."""

from __future__ import annotations

import argparse
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VERSION_RE = re.compile(r'^version\s*=\s*"(?P<version>[^"\n]+)"\s*$', re.MULTILINE)
SEMVER_RE = re.compile(r"^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)$")


@dataclass(frozen=True, order=True)
class Version:
    major: int
    minor: int
    patch: int

    @classmethod
    def parse(cls, value: str) -> "Version":
        match = SEMVER_RE.fullmatch(value)
        if not match:
            raise ValueError(f"unsupported SemVer: {value}")
        return cls(*(int(match.group(name)) for name in ("major", "minor", "patch")))

    def bump(self, kind: str) -> "Version":
        if kind == "patch":
            return Version(self.major, self.minor, self.patch + 1)
        if kind == "minor":
            return Version(self.major, self.minor + 1, 0)
        if kind == "major":
            return Version(self.major + 1, 0, 0)
        raise ValueError(f"unsupported bump: {kind}")

    def __str__(self) -> str:
        return f"{self.major}.{self.minor}.{self.patch}"


@dataclass(frozen=True)
class ReleasePlan:
    version: Version
    mode: str
    ref: str
    commit: str


def workspace_version(root: Path = ROOT) -> Version:
    text = (root / "Cargo.toml").read_text()
    match = VERSION_RE.search(text)
    if not match:
        raise ValueError("workspace Cargo.toml has no package version")
    return Version.parse(match.group("version"))


def valid_tags(tags: list[str]) -> list[Version]:
    versions: list[Version] = []
    for tag in tags:
        if tag.startswith("v"):
            try:
                versions.append(Version.parse(tag[1:]))
            except ValueError:
                continue
    return sorted(set(versions))


def plan_release(
    current: Version,
    tags: list[str],
    bump: str,
    release_complete: bool = False,
    release_commit: str = "",
) -> ReleasePlan:
    tag = f"v{current}"
    versions = valid_tags(tags)

    if tag in tags and not release_complete:
        return ReleasePlan(current, "resume", tag, release_commit)

    if tag in tags and release_complete:
        next_version = current.bump(bump)
        return ReleasePlan(next_version, "new", "HEAD", "")

    if versions and current > versions[-1]:
        return ReleasePlan(current, "resume", release_commit or "HEAD", release_commit)

    return ReleasePlan(current.bump(bump), "new", "HEAD", "")


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def release_commit_for(version: Version) -> str:
    subject = f"chore(release): prepare v{version}"
    try:
        return git("log", "--all", "--format=%H", "--fixed-strings", "--grep", subject, "-n", "1")
    except subprocess.CalledProcessError:
        return ""


def update_version(path: Path, version: Version) -> None:
    text = path.read_text()
    replacement = f'version = "{version}"'
    updated, count = VERSION_RE.subn(replacement, text, count=1)
    if count != 1:
        raise ValueError(f"expected one workspace version in {path}")
    path.write_text(updated)


def bump_manifests(version: Version, root: Path = ROOT) -> None:
    update_version(root / "Cargo.toml", version)
    cli_manifest = root / "crates/workshop-rs-cli/Cargo.toml"
    text = cli_manifest.read_text()
    dependency = re.compile(
        r'(workshop-rs\s*=\s*\{\s*path\s*=\s*"\.\./workshop-rs",\s*version\s*=\s*")[^"]+("\s*\})'
    )
    updated, count = dependency.subn(rf"\g<1>{version}\g<2>", text, count=1)
    if count != 1:
        raise ValueError("workshop-rs-cli has no publishable workshop-rs path/version dependency")
    cli_manifest.write_text(updated)


def print_plan(plan: ReleasePlan) -> None:
    print(f"version={plan.version}")
    print(f"mode={plan.mode}")
    ref = git("rev-parse", "HEAD") if plan.ref == "HEAD" else plan.ref
    print(f"ref={ref}")
    print(f"commit={plan.commit}")


def main() -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)

    plan_parser = subparsers.add_parser("plan")
    plan_parser.add_argument("--bump", choices=("patch", "minor", "major"), default="patch")
    plan_parser.add_argument("--release-complete", action="store_true")

    bump_parser = subparsers.add_parser("bump")
    bump_parser.add_argument("--version", required=True)

    args = parser.parse_args()
    if args.command == "plan":
        current = workspace_version()
        tags = git("tag", "--list", "v*").splitlines()
        commit = release_commit_for(current)
        print_plan(plan_release(current, tags, args.bump, args.release_complete, commit))
    elif args.command == "bump":
        bump_manifests(Version.parse(args.version))


if __name__ == "__main__":
    main()
