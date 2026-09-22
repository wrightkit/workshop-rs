# Strings & Localization

[← Back to Language Support Matrix](../language-support.md)

## String Features

| Feature | Status | Notes |
| --- | --- | --- |
| `Custom String` format strings | ✅ Supported | Format strings with up to 3 interpolation placeholders (`{0}`, `{1}`, `{2}`) and recursive formatting. |
| `String` (Built-in localized string values) | ✅ Supported | Localized preset identities with 0–4 supplied arguments; the preset defaults to `Hello` and replacement arguments default to `Null`, while unsupported preset text is rejected. `Custom String` format strings remain fully supported. |

## Client Locales

| Feature | Status | Notes |
| --- | --- | --- |
| `en-US`, `de-DE`, `es-ES`, `es-MX`, `fr-FR`, `it-IT`, `ja-JP`, `ko-KR`, `pl-PL`, `pt-BR`, `ru-RU`, `th-TH`, `tr-TR`, `zh-CN`, `zh-TW` | ✅ Supported | Declared from the pinned OverPy 9.7.10 Workshop language set. Locale spellings are source-attributed catalog/settings data; uncovered identities remain explicit missing mappings. |
| Bidirectional conversion across the declared locales | ✅ Supported | Conversion resolves locale-independent canonical identities and fails explicitly on missing mappings; opt-in fallback is recorded in the conversion result. |
