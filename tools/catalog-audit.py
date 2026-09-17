#!/usr/bin/env python3
"""Audit the Workshop catalog against a provenance-linked Markdown snapshot."""

from __future__ import annotations

import argparse
import json
import re
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen


SCHEMA_VERSION = 1
DEFAULT_CATALOG = "crates/workshop-rs/src/catalog/data/catalog.json"
DEFAULT_SETTINGS = "crates/workshop-rs/src/settings/data/inventory.json"
DEFAULT_REFERENCE = "tools/catalog-reference.json"
DEFAULT_BASE_URL = "https://md.wrightkit.dev"
RELEVANT_CATEGORIES = {"actions", "values", "events", "constants"}


def read_json(path: Path) -> dict:
    try:
        value = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read JSON {path}: {error}") from error
    if not isinstance(value, dict):
        raise ValueError(f"JSON root must be an object: {path}")
    return value


def validate_reference(reference: dict) -> None:
    if reference.get("schemaVersion") != SCHEMA_VERSION:
        raise ValueError(f"unsupported reference snapshot schemaVersion {reference.get('schemaVersion')}")
    source = reference.get("source")
    if not isinstance(source, dict) or not isinstance(source.get("baseUrl"), str):
        raise ValueError("reference snapshot has no source baseUrl")
    facts = reference.get("facts")
    if not isinstance(facts, list):
        raise ValueError("reference snapshot facts must be an array")
    base_url = source["baseUrl"].rstrip("/") + "/"
    for index, fact in enumerate(facts):
        if not isinstance(fact, dict) or not isinstance(fact.get("sourceUrl"), str):
            raise ValueError(f"reference fact {index} has no sourceUrl")
        if not fact["sourceUrl"].startswith(base_url):
            raise ValueError(f"reference fact {index} sourceUrl is outside {source['baseUrl']}")


def aliases(entry: dict) -> list[str]:
    value = entry.get("aliases", {}).get("en-US", [])
    return value if isinstance(value, list) else [value]


def catalog_entries(catalog: dict, settings: dict) -> list[dict]:
    result = []
    for kind in ("structural", "actions", "values", "events", "operators"):
        for entry in catalog.get(kind, []):
            result.append(
                {
                    "kind": kind[:-1] if kind.endswith("s") else kind,
                    "id": entry["id"],
                    "title": aliases(entry)[0],
                    "aliases": aliases(entry),
                    "params": entry.get("params", []),
                    "paramTypes": entry.get("paramTypes", []),
                    "returnType": entry.get("returnType"),
                    "source": f"catalog.{kind}.{entry['id']}",
                }
            )
    for domain in catalog.get("enums", []):
        members = [
            {"id": member["id"], "title": aliases(member)[0], "aliases": aliases(member)}
            for member in domain.get("members", [])
        ]
        result.append(
            {
                "kind": "enum",
                "id": domain["domain"],
                "title": aliases(domain)[0] if aliases(domain) else domain["domain"],
                "aliases": aliases(domain),
                "members": members,
                "source": f"catalog.enums.{domain['domain']}",
            }
        )
        result.extend(
            {
                "kind": "enumMember",
                "id": f"{domain['domain']}.{member['id']}",
                "title": aliases(member)[0],
                "aliases": aliases(member),
                "domain": domain["domain"],
                "source": f"catalog.enums.{domain['domain']}.{member['id']}",
            }
            for member in domain.get("members", [])
        )
    for entry in catalog.get("localizedStrings", []):
        result.append(
            {
                "kind": "localizedString",
                "id": entry["id"],
                "title": aliases(entry)[0],
                "aliases": aliases(entry),
                "source": f"catalog.localizedStrings.{entry['id']}",
            }
        )
    for section, entries in settings.get("entries", {}).items():
        for entry in entries:
            result.append(
                {
                    "kind": "setting",
                    "id": f"{section}/{entry['id']}",
                    "title": entry["en-US"],
                    "aliases": [entry["en-US"]],
                    "source": entry.get("source", f"settings.{section}.{entry['id']}"),
                }
            )
    return result


def _find_current(current: list[dict], fact: dict) -> dict | None:
    identity = fact.get("id")
    same_id = [entry for entry in current if entry["id"] == identity]
    if fact.get("kind") == "enum" and fact.get("domain"):
        same_id = [entry for entry in current if entry["id"] == fact["domain"]]
    if same_id:
        return same_id[0]
    title = fact.get("title")
    if title:
        titled = [entry for entry in current if title in entry.get("aliases", [])]
        if len(titled) == 1:
            return titled[0]
    return None


def _finding(entry: dict | None, mismatch: str, expected, current, source: str | None, status: str) -> dict:
    return {
        "status": status,
        "mismatch": mismatch,
        "entry": entry.get("source") if entry else None,
        "entryId": entry.get("id") if entry else None,
        "expected": expected,
        "current": current,
        "sourceUrl": source,
    }


def _type_name(value: str | None) -> str | None:
    if value is None:
        return None
    return re.sub(r"\s*\|\s*", "|", value.strip()).replace("Integer", "Number").replace("Float", "Number")


def audit(catalog: dict, reference: dict, settings: dict) -> dict:
    current = catalog_entries(catalog, settings)
    facts = reference.get("facts", [])
    findings: list[dict] = []
    matched: set[int] = set()
    mismatched: set[int] = set()

    for fact in facts:
        source = fact.get("sourceUrl")
        if fact.get("status") in {"conflicting", "insufficient"}:
            findings.append(
                _finding(None, "conflicting or insufficient reference evidence", fact.get("status"), None, source, "unverified")
            )
            continue
        if fact.get("kind") == "operation":
            operators = {entry["title"] for entry in current if entry["kind"] == "operator"}
            expected = set(fact.get("members", []))
            for title in sorted(expected - operators):
                findings.append(_finding(None, "enum/operation membership mismatch", title, None, source, "reference-only"))
            for title in sorted(operators - expected):
                entry = next(entry for entry in current if entry["kind"] == "operator" and entry["title"] == title)
                is_known_value = any(candidate["title"] == title and candidate["kind"] == "value" for candidate in current)
                findings.append(_finding(entry, "enum/operation membership mismatch", None, title, source, "mismatch" if is_known_value else "unverified"))
            continue

        entry = _find_current(current, fact)
        if entry is None:
            findings.append(_finding(None, "present in reference but missing from catalog", fact, None, source, "reference-only"))
            continue
        matched.add(id(entry))
        if fact.get("kind") and fact["kind"] != entry["kind"]:
            findings.append(_finding(entry, "wrong catalog kind/classification", fact["kind"], entry["kind"], source, "mismatch"))
        if fact.get("title") and fact["title"] != entry["title"]:
            findings.append(_finding(entry, "alias/spelling mismatch", fact["title"], entry["title"], source, "mismatch"))
        expected_aliases = fact.get("aliases")
        if expected_aliases is not None and set(expected_aliases) != set(entry.get("aliases", [])):
            findings.append(_finding(entry, "alias/spelling mismatch", expected_aliases, entry.get("aliases", []), source, "mismatch"))
        if "params" in fact:
            expected_params = fact["params"]
            actual_params = entry.get("params", [])
            if len(expected_params) != len(actual_params):
                findings.append(_finding(entry, "signature/arity mismatch", len(expected_params), len(actual_params), source, "mismatch"))
            elif expected_params != actual_params:
                findings.append(_finding(entry, "signature/arity mismatch", expected_params, actual_params, source, "mismatch"))
        expected_types = [_type_name(value) for value in fact.get("paramTypes", [])]
        actual_types = [_type_name(value) for value in entry.get("paramTypes", [])]
        if "paramTypes" in fact and expected_types != actual_types:
            findings.append(_finding(entry, "parameter type/domain mismatch", fact["paramTypes"], entry.get("paramTypes", []), source, "mismatch"))
        if "returnType" in fact and _type_name(fact["returnType"]) != _type_name(entry.get("returnType")):
            findings.append(_finding(entry, "return-type mismatch", fact["returnType"], entry.get("returnType"), source, "mismatch"))
        if "members" in fact:
            expected_members = [(member["id"], member["title"]) for member in fact["members"]]
            actual_members = [(member["id"], member["title"]) for member in entry.get("members", [])]
            if expected_members != actual_members:
                findings.append(_finding(entry, "enum/operation membership mismatch", expected_members, actual_members, source, "mismatch"))
                expected_by_id = dict(expected_members)
                for member in current:
                    if member.get("domain") == entry["id"]:
                        if expected_by_id.get(member["id"].split(".", 1)[-1]) == member["title"]:
                            matched.add(id(member))
                        else:
                            mismatched.add(id(member))
            else:
                for member in current:
                    if member.get("domain") == entry["id"]:
                        matched.add(id(member))

    for entry in current:
        if id(entry) not in matched and not any(
            finding.get("entryId") == entry["id"] and finding["status"] == "mismatch" for finding in findings
        ):
            findings.append(
                _finding(entry, "present in catalog but not evidenced by the reference", None, entry, None, "unverified")
            )

    surfaces = {}
    for entry in current:
        surface = entry["kind"]
        row = surfaces.setdefault(surface, {"catalog": 0, "evidenced": 0, "unverified": 0})
        row["catalog"] += 1
        if id(entry) in matched:
            row["evidenced"] += 1
        else:
            row["unverified"] += 1
    for finding in findings:
        if finding["status"] == "reference-only":
            surface = "reference"
            surfaces.setdefault(surface, {"catalog": 0, "evidenced": 0, "unverified": 0})["unverified"] += 1

    hard = [finding for finding in findings if finding["status"] in {"mismatch", "reference-only"}]
    entry_coverage = []
    for entry in current:
        entry_findings = [finding for finding in findings if finding.get("entry") == entry["source"]]
        entry_coverage.append({
            "entry": entry["source"],
            "kind": entry["kind"],
            "id": entry["id"],
            "status": "mismatch" if id(entry) in mismatched or any(finding["status"] == "mismatch" for finding in entry_findings) else "evidenced" if id(entry) in matched else "unverified",
        })
    return {
        "schemaVersion": SCHEMA_VERSION,
        "status": "pass" if not hard else "mismatch",
        "reference": reference.get("source", {}),
        "coverage": {
            "catalogEntries": len(current),
            "referenceFacts": len(facts),
            "surfaces": surfaces,
            "entries": entry_coverage,
        },
        "findings": findings,
    }


def _fetch(url: str) -> str:
    request = Request(url, headers={"Accept": "text/markdown, application/json", "User-Agent": "workshop-rs-catalog-audit/1"})
    try:
        with urlopen(request, timeout=30) as response:
            return response.read().decode("utf-8")
    except (HTTPError, URLError, TimeoutError) as error:
        raise RuntimeError(f"failed to fetch {url}: {error}") from error


def _markdown_fact(markdown: str, document: dict, catalog: dict) -> dict | None:
    category = str(document.get("category", "")).lower()
    title = document.get("title", "")
    current_matches = [
        entry
        for entry in catalog_entries(catalog, {"entries": {}})
        if title in entry.get("aliases", [])
    ]
    if category not in RELEVANT_CATEGORIES and title != "Operation" and not current_matches:
        return None
    source = document.get("markdownUrl") or document.get("sourceUrl")
    if len(current_matches) > 1:
        return {
            "id": document.get("slug", title),
            "title": title,
            "status": "conflicting",
            "candidates": [entry["source"] for entry in current_matches],
            "sourceUrl": source,
        }
    matched = current_matches[0] if len(current_matches) == 1 else None
    fact = {"id": matched["id"] if matched else document.get("slug", title), "title": title, "sourceUrl": source}
    if category in {"actions", "values", "events"}:
        fact["kind"] = category[:-1]
    elif matched and matched["kind"] in {"action", "value", "event", "operator", "structural"}:
        fact["kind"] = matched["kind"]
    elif title == "Operation":
        fact["kind"] = "operation"
    else:
        fact["kind"] = "enum"
    returns = re.search(r"\*\*Returns\*\*:\s*_?(?:<[^>]+>)*([^_\n]+)", markdown)
    if returns:
        fact["returnType"] = re.sub(r"<[^>]+>", "", returns.group(1)).strip()
    content_hash = re.search(r"^content_hash:\s*([0-9a-f]{64})$", markdown, re.MULTILINE)
    if content_hash:
        fact["contentHash"] = content_hash.group(1)
    parameters = re.search(r"\*\*Parameters\*\*:\s*(.*)", markdown)
    if parameters:
        fact["params"] = re.findall(r"==_([^_]+)_==", parameters.group(1))
    detailed = re.findall(r"> ==\*\*([^*]+)\*\*==\s*\n> _Type: \*\*([^*]+)\*\*", markdown)
    if detailed:
        fact["params"] = [name for name, _ in detailed]
        fact["paramTypes"] = [value for _, value in detailed]
    if fact["kind"] == "operation":
        options = markdown.split("## Options", 1)[-1]
        fact["members"] = re.findall(r"^- \*\*([^*]+)\*\*", options, re.MULTILINE)
    if fact["kind"] == "enum":
        options = markdown.split("Options", 1)[-1].split("##", 1)[0]
        fact["members"] = [{"id": option, "title": option, "aliases": [option]} for option in re.findall(r"^- \*\*([^*]+)\*\*", options, re.MULTILINE)]
    return fact


def refresh(base_url: str, catalog_path: Path, output: Path) -> None:
    manifest_url = base_url.rstrip("/") + "/manifest.json"
    manifest = json.loads(_fetch(manifest_url))
    if manifest.get("schemaVersion") != 1 or not isinstance(manifest.get("documents"), list):
        raise RuntimeError(f"unsupported manifest from {manifest_url}")
    documents = manifest["documents"]
    catalog_data = read_json(catalog_path)
    selected = [
        document
        for document in documents
        if not document.get("category")
        or str(document.get("category", "")).lower() in RELEVANT_CATEGORIES
        or document.get("title") == "Operation"
    ]
    facts = []
    errors = []
    with ThreadPoolExecutor(max_workers=12) as pool:
        pending = {pool.submit(_fetch, document["markdownUrl"]): document for document in selected}
        for future in as_completed(pending):
            document = pending[future]
            try:
                fact = _markdown_fact(future.result(), document, catalog_data)
                if fact:
                    facts.append(fact)
            except RuntimeError as error:
                errors.append(str(error))
    if errors:
        raise RuntimeError(f"reference refresh incomplete ({len(errors)} fetches failed); first: {errors[0]}")
    snapshot = {
        "schemaVersion": SCHEMA_VERSION,
        "source": {"baseUrl": base_url, "manifestUrl": manifest_url, "manifestSchemaVersion": 1},
        "documents": [
            {key: document[key] for key in ("title", "slug", "markdownUrl", "sourceUrl", "updatedAt") if key in document}
            for document in sorted(documents, key=lambda item: item.get("slug", ""))
        ],
        "facts": sorted(facts, key=lambda item: (item.get("kind", ""), item.get("id", ""))),
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(snapshot, ensure_ascii=False, indent=2) + "\n")
    print(f"wrote {output} ({len(facts)} facts, {len(documents)} documents)")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    verify_parser = subparsers.add_parser("verify")
    verify_parser.add_argument("--catalog", type=Path, default=Path(DEFAULT_CATALOG))
    verify_parser.add_argument("--settings", type=Path, default=Path(DEFAULT_SETTINGS))
    verify_parser.add_argument("--reference", type=Path, default=Path(DEFAULT_REFERENCE))
    verify_parser.add_argument("--json", action="store_true")
    verify_parser.add_argument("--strict", action="store_true", help="also fail on explicit unverified coverage")
    refresh_parser = subparsers.add_parser("refresh")
    refresh_parser.add_argument("--base-url", default=DEFAULT_BASE_URL)
    refresh_parser.add_argument("--catalog", type=Path, default=Path(DEFAULT_CATALOG))
    refresh_parser.add_argument("--output", type=Path, default=Path(DEFAULT_REFERENCE))
    args = parser.parse_args(argv)
    try:
        if args.command == "refresh":
            refresh(args.base_url, args.catalog, args.output)
            return 0
        reference = read_json(args.reference)
        validate_reference(reference)
        result = audit(read_json(args.catalog), reference, read_json(args.settings))
        if args.json:
            print(json.dumps(result, ensure_ascii=False, indent=2))
        else:
            print(f"{result['status'].upper()}: {result['coverage']['catalogEntries']} catalog entries, {result['coverage']['referenceFacts']} reference facts")
            for surface, counts in sorted(result["coverage"]["surfaces"].items()):
                print(f"{surface}: catalog={counts['catalog']} evidenced={counts['evidenced']} unverified={counts['unverified']}")
            for finding in result["findings"]:
                if finding["status"] in {"mismatch", "reference-only"}:
                    print(f"{finding['status']}: {finding['mismatch']}: {finding.get('entry') or finding.get('entryId') or '<reference>'}")
        return 1 if result["status"] != "pass" or (args.strict and any(f["status"] == "unverified" for f in result["findings"])) else 0
    except (OSError, ValueError, RuntimeError) as error:
        print(f"catalog-audit: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
