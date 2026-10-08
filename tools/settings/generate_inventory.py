#!/usr/bin/env python3
"""Generate the machine-readable settings inventory from workshop-data.

The source export is intentionally external to this repository.  The output
is deterministic and records the source commit, input digest, and source path
for every discovered localized label.  Rust settings tables remain the
checked-in consumer projection; `workshop-catalog-gen check` validates that
projection against the canonical catalog.
"""

import argparse
import hashlib
import json
from pathlib import Path

# The inventory records the catalog's working locales: canonical en-US
# spellings plus the zh-CN corpus translations.
LOCALES = ("en-US", "zh-CN")


def localized_aliases(value):
    return {
        key: child
        for key, child in value.items()
        if key in LOCALES and isinstance(child, str)
    }


def labeled_record(aliases, source, identifier=None):
    record = {}
    if identifier is not None:
        record["id"] = identifier
    if "en-US" in aliases:
        record["en-US"] = aliases["en-US"]
    record["source"] = source
    if "zh-CN" in aliases:
        record["zh-CN"] = aliases["zh-CN"]
    return record


def walk_labels(value, path, out):
    if isinstance(value, dict):
        for key, child in value.items():
            aliases = localized_aliases(child) if isinstance(child, dict) else {}
            if aliases:
                out.append(labeled_record(aliases, ".".join((*path, key))))
            else:
                walk_labels(child, (*path, key), out)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            walk_labels(child, (*path, str(index)), out)


def top_level_entries(data, category):
    result = []
    for key, item in data[category].items():
        if not isinstance(item, dict):
            continue
        aliases = localized_aliases(item)
        if not aliases:
            continue
        result.append(labeled_record(aliases, f"data.{category}.{key}", identifier=key))
    return sorted(result, key=lambda item: item["id"])


def reject_alias_conflicts(entries, category, locales):
    by_alias = {}
    for locale in locales:
        for entry in entries:
            alias = entry.get(locale)
            if alias:
                by_alias.setdefault((locale, alias), []).append(entry["id"])
    conflicts = {
        f"{locale}:{alias}": ids
        for (locale, alias), ids in by_alias.items()
        if len(ids) > 1
    }
    if conflicts:
        raise SystemExit(f"{category}: duplicate localized aliases: {conflicts}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--export", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--expected-commit", default="5a7d0e294b8cad73b9701987bb584d0551d7fa4d")
    args = parser.parse_args()

    raw = args.export.read_bytes()
    document = json.loads(raw)
    meta = document["meta"]
    if meta["commit"] != args.expected_commit:
        raise SystemExit(
            f"unexpected workshop-data commit {meta['commit']}; expected {args.expected_commit}"
        )
    settings_groups = []
    walk_labels(document["data"].get("customGameSettings", {}), ("data", "customGameSettings"), settings_groups)
    entries = {
        category: top_level_entries(document["data"], category)
        for category in ("values", "gamemodes", "maps", "heroes")
    }
    locales = sorted(
        {
            locale
            for entry in settings_groups
            for locale in entry
            if locale != "source"
        }
        | {
            locale
            for category_entries in entries.values()
            for entry in category_entries
            for locale in entry
            if locale not in {"id", "source"}
        }
    )
    settings_groups.sort(key=lambda item: (item.get(locales[0], ""), item["source"]))
    for category, category_entries in entries.items():
        reject_alias_conflicts(category_entries, category, locales)
    output = {
        "schemaVersion": 1,
        "source": {
            "commit": meta["commit"],
            "commitDate": meta["commitDate"],
            "sha256": hashlib.sha256(raw).hexdigest(),
        },
        "counts": {
            "actions": len(document["data"]["actions"]),
            "values": len(document["data"]["values"]),
            "gamemodes": len(document["data"]["gamemodes"]),
            "maps": len(document["data"]["maps"]),
            "heroes": len(document["data"]["heroes"]),
            "settingsGroups": len(settings_groups),
        },
        "entries": entries,
        "settingsGroups": settings_groups,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, ensure_ascii=False, indent=2) + "\n")


if __name__ == "__main__":
    main()
