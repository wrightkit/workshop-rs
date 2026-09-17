# Catalog reference audit

`tools/catalog-audit.py` compares the canonical catalog with reviewed facts
extracted from the Markdown reference at `md.wrightkit.dev`. The committed
`tools/catalog-reference.json` is an offline snapshot. It is evidence, not a
replacement for canonical catalog data, and it never rewrites catalog
semantics.

Run the deterministic check from the repository root:

```sh
python3 tools/catalog-audit.py verify
python3 tools/catalog-audit.py verify --json
```

Every catalog entry, including structural entries, localized strings, enum
domains, settings inventory entries, and entries not covered by the snapshot,
appears in `coverage.entries`. An uncovered entry is `unverified`; it is not
silently treated as a reference absence. Hard findings fail the command:

- `mismatch` means an evidenced fact conflicts with the catalog;
- `reference-only` means an evidenced reference fact has no catalog entry;
- `unverified` means the reference does not establish the fact.

The audit distinguishes kind, spelling, arity/signature, parameter type/domain,
return type, enum/operation membership, and insufficient/conflicting evidence.
Type aliases such as `Integer`/`Float` versus the catalog's `Number` are
normalized only for comparison.

Refresh is an explicitly networked operation and writes a new reviewed
snapshot only after all selected Markdown documents are fetched:

```sh
python3 tools/catalog-audit.py refresh
```

The refresh path uses the manifest and exact Markdown URLs, records those URLs
on every extracted fact, and aborts on partial acquisition. Ordinary CI only
consumes the committed snapshot and does not require network access.
