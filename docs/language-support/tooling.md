# Tooling & Semantic Capabilities

[← Back to Language Support Matrix](../language-support.md)

## Compiler & Tooling Surfaces

| Feature | Status | Notes |
| --- | --- | --- |
| Raw Workshop parsing | ✅ Supported | Parses raw text into syntax trees with source location spans. |
| Semantic validation | ✅ Supported | Validates arity, parameter types, enum domains, and variable/subroutine declarations. |
| Deterministic code emission | ✅ Supported | Formats and emits canonical Workshop code deterministically for supported locales. |
| Workshop language conversion | ✅ Supported | Bidirectional conversion across the 15 declared client locales, with explicit missing-mapping errors and recorded opt-in fallback. |
| Hero gameplay & semantic query API | ✅ Supported | Query hero abilities, slots, variants, custom-game modifiers, and cooldown calculations. |
| Canonical name lookup | ✅ Supported | `Catalog::lookup` resolves display names, near spellings, and settings paths to canonical identities, signatures, enum domains, and settings keys; `unknown ... spelling` diagnostics carry nearest candidates. |
| Offline feature census & conformance testing | ✅ Supported | Sharded offline census and regression runner for compatibility tracking. |
