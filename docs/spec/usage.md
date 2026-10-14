# Library and CLI usage

## CLI

```sh
workshop-rs-cli parse file.ws
workshop-rs-cli emit file.ws
workshop-rs-cli convert file.ws --from en-US --to zh-CN
workshop-rs-cli convert file.ws --from en-US --to zh-CN --fallback-locale en-US
workshop-rs-cli locales
workshop-rs-cli version --json
workshop-rs-cli census [--json]
workshop-rs-cli corpus manifest.json [--json]
workshop-rs-cli seasonal-diff previous.json current.json [--json]
```

Exit codes: `0` success, `1` parse/emit/conversion/catalog failure, `2` usage
error.

## Library

```rust
use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::convert::{convert, ConvertOptions};
use workshop_rs::emitter::emit;

let catalog = Catalog::builtin()?;
let locale = Locale::new("en-US");
let program = workshop_rs::parser::parse(text, &catalog, &locale)?;
program.validate()?;
workshop_rs::validate::validate_canonical_ids(&program, &catalog)?;
let emitted = emit(&program, &catalog, &locale)?;

// An independent language frontend can lower directly into the same model.
let mut generated = workshop_rs::Program::new();
generated
    .global_variable(workshop_rs::Variable::new("Score"))
    .rule(workshop_rs::Rule::new("generated", workshop_rs::Event::Global)
        .action(workshop_rs::Action::SetGlobalVariable {
            variable: "Score".to_string(),
            value: workshop_rs::Value::number(1.0),
        }));
let generated_text = emit(&generated, &catalog, &locale)?;

let converted = convert(
    text,
    &catalog,
    &Locale::new("en-US"),
    &Locale::new("zh-CN"),
    &ConvertOptions::default(),
)?;
```

## Catalog data pipeline

Catalog and gameplay dataset updates are reviewed data changes verified by
deterministic generators:

```sh
cargo run -p workshop-rs --bin workshop-catalog-gen -- check
cargo run -p workshop-rs --bin workshop-catalog-gen -- build
```

Source-attribution requirements are in
[`../source-attribution/README.md`](../source-attribution/README.md).
