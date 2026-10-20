# Practice Range real-project fixtures

`en-US.ow` and `zh-CN.ow` are pinned copies of
`generated/<locale>/practice-range.ow` from the
[`wrightkit/practice-range`](https://github.com/wrightkit/practice-range)
repository at revision
`6e5ecc6904473de397585e3bf2ff5f815efc3394`, which is generated from the
`workshop-rs` catalog and committed by that repository. The corpus covers
every action, value, operator, enum member, and event the catalog declares,
so it exercises the full surface `rule-content-v1` must express — including
the unpinned bare enum members (`Up`, `Down`, `Left`, `Right` in
`setGlobalVariable` and `createIcon` argument positions) that the parser
preserves as ambiguous.

| Fixture | Source path | Locale | SHA-256 |
| --- | --- | --- | --- |
| `en-US.ow` | `generated/en-US/practice-range.ow` | en-US | `9f1ef1328e923f205f09a8a006d40be47defac2c7c92416e8091134925fca07a` |
| `zh-CN.ow` | `generated/zh-CN/practice-range.ow` | zh-CN | `b6e099c29068b7b2e0664c67ddd17d4dec51d1b2b2084acb50a30a96e1e3b98c` |

These are generated artifacts of a WrightKit repository vendored as a
test input; refresh them deliberately from a recorded practice-range
revision rather than editing them in place.
