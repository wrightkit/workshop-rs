# workshop-rs

`workshop-rs` is a Rust library and CLI for raw Overwatch Workshop text. It
parses, validates, converts, queries, and emits Workshop code, and it is the
canonical Workshop core shared by [`opy-rs`](https://github.com/wrightkit/opy-rs),
[`deltin-rs`](https://github.com/wrightkit/deltin-rs),
[`athena`](https://github.com/wrightkit/athena), and
[Wright](https://github.com/wrightkit/wright).

## Features

- **Parse and emit** raw Workshop with a shared `Program` model.
- **Validate** actions, values, events, enums, and settings against a built-in
  catalog.
- **Convert** between every declared Workshop client locale, with strict checks
  for missing terms.
- **Preserve source**: authored text and comments stay attached to the parsed
  program for checked edits and source mapping.
- **Query** hero abilities, weapon slots, custom game modifiers, versioned hero
  values, and element counts.
- **Look up** canonical names and near-miss spellings in the catalog.
- **Verification tools** for feature census, regression corpora, and client
  version drift.

## Install

```sh
cargo add workshop-rs            # library
cargo install workshop-rs-cli    # CLI
```

Precompiled CLI archives are attached to
[GitHub Releases](https://github.com/wrightkit/workshop-rs/releases).

## Quick start

```sh
workshop-rs-cli parse file.ws
workshop-rs-cli emit file.ws
workshop-rs-cli convert file.ws --from en-US --to zh-CN
workshop-rs-cli locales
```

Library usage and the full CLI reference are in
[`docs/spec/usage.md`](docs/spec/usage.md).

## Language support

The declared Workshop capability surface is listed in
[`docs/language-support.md`](docs/language-support.md).

## Documentation

See [`docs/README.md`](docs/README.md) for architecture, data contracts, and
release procedures.

## Contributing

Read [`AGENTS.md`](AGENTS.md) first.

## License

MIT. Dataset and fixture source attribution is in
[`docs/source-attribution/README.md`](docs/source-attribution/README.md).
