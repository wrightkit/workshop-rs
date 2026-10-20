#!/usr/bin/env python3
"""Extract the zh-CN description sources for `workshop-catalog-gen descriptions`.

Reads the two reviewed zh-CN sources and writes one normalized JSON document
that the catalog pipeline consumes via `--zh-source`:

- `owlCN`: the zh-CN Workshop client strings in the owl-CN extension
  (`client/owl.js`), keyed by the zh-CN element name. `owl.js` is JavaScript,
  not JSON, so it is evaluated with Node while `vscode` and `pinyin` requires
  are stubbed; the extension's merged `keywordObj` (actions plus every value
  table) is the complete keyword map.
- `owmod`: the docs.owmod.net Chinese manual (GKD-OW/docs), parsed from
  `manual/documents/*.md`, keyed by the English name each `## zh  En` heading
  carries. Parameter bullets are `- `label` text` lines.

Usage:
    extract_zh_source.py --owl <owl-CN checkout> --owmod <GKD-OW docs checkout> --out zh-source.json

Both checkouts must sit at the commits pinned in
docs/source-attribution/catalog.md. The output records each source's
repository, resolved commit, source path, and sha256.
"""

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

# Node snippet: stub the extension host requires, load owl.js, dump the
# merged keyword table. `keywordObj` merges the action table and every value
# table (variables, IDs, boolean, numbers, others, players, vectors, angles,
# arrays, strings, colors, update230412); `constants` holds enum members and
# is not keyword data.
OWL_SNIPPET = """
// owl.js is an extension host script: it calls the vscode API at module
// load time. A permissive Proxy stubs every member as a callable
// constructor/function so the keyword data tables load unchanged.
const vscode = new Proxy(function () {}, {
  get: (target, prop) => (prop === Symbol.toPrimitive ? () => 0 : vscode),
  construct: () => vscode,
  apply: () => vscode,
});
const Module = require('module');
const orig = Module._load;
Module._load = function(request, parent, isMain) {
  if (request === 'vscode') return vscode;
  if (request.endsWith('pinyin.js')) return { Pinyin: {} };
  return orig.apply(this, arguments);
};
// owl.js logs a pinyin-mismatch notice on stdout at load time, so the
// keyword table is written to the path in argv[2] instead of stdout.
const owl = require(process.argv[1]);
require('fs').writeFileSync(process.argv[2], JSON.stringify(owl.keywordObj));
"""


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_commit(path: Path) -> str:
    result = subprocess.run(
        ["git", "-C", str(path), "rev-parse", "HEAD"],
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout.strip()


def extract_owl(repo: Path, tmpdir: Path) -> tuple[dict, dict]:
    owl_js = repo / "client" / "owl.js"
    dump = tmpdir / "owl-keywords.json"
    subprocess.run(
        ["node", "-e", OWL_SNIPPET, str(owl_js.resolve()), str(dump)],
        capture_output=True,
        text=True,
        check=True,
    )
    keywords = json.loads(dump.read_text(encoding="utf-8"))
    entries = {}
    for name, record in keywords.items():
        description = (record.get("描述") or "").strip()
        params = [
            {
                "name": param.get("名称") or "",
                "description": (param.get("说明") or "").strip(),
            }
            for param in record.get("参数") or []
        ]
        entries[name] = {"description": description, "params": params}
    source = {
        "repo": "https://github.com/qaz075115961/owl-CN",
        "commit": git_commit(repo),
        "path": "client/owl.js",
        "sha256": sha256_file(owl_js),
    }
    return entries, source


def extract_owmod(repo: Path) -> tuple[dict, dict]:
    documents = sorted((repo / "manual" / "documents").glob("*.md"))
    if not documents:
        sys.exit(f"no manual documents under {repo}/manual/documents")
    entries = {}
    digest = hashlib.sha256()
    for path in documents:
        digest.update(path.read_bytes())
        current = None
        for line in path.read_text(encoding="utf-8").splitlines():
            heading = re.match(r"^##\s+(.+?)\s{2,}([A-Za-z].*?)\s*$", line)
            if heading:
                current = {
                    "zh": heading.group(1).strip(),
                    "description": "",
                    "params": [],
                }
                entries[heading.group(2).strip()] = current
                continue
            if line.startswith("##"):
                current = None
                continue
            if current is None:
                continue
            stripped = line.strip()
            if (
                not current["description"]
                and stripped.startswith("_")
                and stripped.endswith("_")
                and len(stripped) > 2
            ):
                current["description"] = stripped.strip("_")
                continue
            param = re.match(r"^-\s*`([^`]+)`\s*(.*)$", stripped)
            if param:
                current["params"].append(
                    {"name": param.group(1), "description": param.group(2)}
                )
    source = {
        "repo": "https://github.com/GKD-OW/docs",
        "commit": git_commit(repo),
        "path": "manual/documents",
        "sha256": digest.hexdigest(),
    }
    return entries, source


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--owl", required=True, type=Path, help="owl-CN checkout")
    parser.add_argument("--owmod", required=True, type=Path, help="GKD-OW docs checkout")
    parser.add_argument("--out", required=True, type=Path, help="output JSON path")
    args = parser.parse_args()

    import tempfile

    with tempfile.TemporaryDirectory() as tmpdir:
        owl_entries, owl_source = extract_owl(args.owl, Path(tmpdir))
    owmod_entries, owmod_source = extract_owmod(args.owmod)
    document = {
        "sources": {"owlCN": owl_source, "owmod": owmod_source},
        "owlCN": owl_entries,
        "owmod": owmod_entries,
    }
    args.out.write_text(
        json.dumps(document, ensure_ascii=False, indent=1, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(
        f"wrote {args.out}: {len(owl_entries)} owl-CN keywords, "
        f"{len(owmod_entries)} owmod entries"
    )


if __name__ == "__main__":
    main()
