# Strings & Localization

[← Back to Language Support Matrix](../language-support.md)

## String Features

| Feature | Status | Notes |
| --- | --- | --- |
| `Custom String` format strings | ✅ Supported | Format strings with up to 3 interpolation placeholders (`{0}`, `{1}`, `{2}`) and recursive formatting. |
| Built-in localized `String` values | ✅ Supported | Standard localized Workshop preset string identifiers. |

## Client Locales

| Feature | Status | Notes |
| --- | --- | --- |
| `en-US` client locale | ✅ Supported | Primary locale with 100% complete catalog and syntax coverage for parsing, emission, and conversion. |
| `zh-CN` client locale | ✅ Supported | Reviewed canonical localization with high coverage for parsing, emission, and conversion. |
| Additional client locales (`ko-KR`, `ja-JP`, `de-DE`, `fr-FR`, `es-ES`, etc.) | 🚧 Coming soon | Planned for addition upon ingestion of provenance-reviewed game client datasets. |
| Bidirectional conversion (`en-US` ↔ `zh-CN`) | ✅ Supported | Strict conversion with explicit error on unmapped identities, plus opt-in fallback to primary locale. |
