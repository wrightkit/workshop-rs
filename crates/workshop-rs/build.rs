use std::{collections::HashSet, env, fmt::Write, fs, path::PathBuf};

use serde_json::Value;

fn main() {
    println!("cargo:rerun-if-changed=src/catalog/data/catalog.json");

    let catalog_path = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("src/catalog/data/catalog.json");
    let catalog = fs::read_to_string(&catalog_path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", catalog_path.display()));
    let catalog: Value = serde_json::from_str(&catalog)
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", catalog_path.display()));

    let mut generated = String::new();
    generate_impl(
        &mut generated,
        &catalog,
        "Action",
        "actions",
        &["call", "disabled"],
    );
    generate_impl(
        &mut generated,
        &catalog,
        "Value",
        "values",
        &[
            "call",
            "number",
            "string",
            "global_variable",
            "player_variable",
        ],
    );

    let output_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("typed_api.rs");
    fs::write(&output_path, generated)
        .unwrap_or_else(|error| panic!("cannot write {}: {error}", output_path.display()));
}

fn generate_impl(
    output: &mut String,
    catalog: &Value,
    type_name: &str,
    section: &str,
    reserved: &[&str],
) {
    let entries = catalog
        .get(section)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("catalog section '{section}' is not an array"));

    writeln!(output, "impl {type_name} {{").unwrap();
    let mut generated_names = Vec::new();
    for entry in entries {
        let id = entry
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("{section} entry has no string id"));
        let method_name = method_name(type_name, id);
        if reserved.contains(&method_name.as_str()) {
            panic!("catalog {section} id '{id}' conflicts with {type_name}::{method_name}");
        }
        if generated_names.iter().any(|name| name == &method_name) {
            panic!("catalog {section} ids collide as {type_name}::{method_name}");
        }
        generated_names.push(method_name.clone());

        let params = entry
            .get("params")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let variadic = entry
            .get("variadic")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let mut used_parameter_names = HashSet::new();
        let parameter_names: Vec<String> = params
            .iter()
            .enumerate()
            .map(|(index, parameter)| {
                let parameter = parameter.as_str().unwrap_or_else(|| {
                    panic!("{section} entry '{id}' has a non-string parameter name")
                });
                let base = identifier(parameter, &format!("arg_{index}"));
                let mut name = base.clone();
                let mut suffix = 2;
                while !used_parameter_names.insert(name.clone()) {
                    name = format!("{base}_{suffix}");
                    suffix += 1;
                }
                name
            })
            .collect();

        let description = format!("Constructs the canonical Workshop {section} `{id}`.");
        writeln!(output, "    #[doc = {:?}]", description).unwrap();
        if !parameter_names.is_empty() {
            writeln!(
                output,
                "    #[doc = \"Parameters: {}.\"]",
                parameter_names.join(", ")
            )
            .unwrap();
        }
        if variadic {
            if parameter_names.len() != 1 {
                panic!("catalog variadic {section} entry '{id}' must have one repeated parameter");
            }
            writeln!(
                output,
                "    pub fn {method_name}(values: impl IntoIterator<Item = impl Into<Value>>) -> Self {{"
            )
            .unwrap();
            writeln!(
                output,
                "        Self::call({id:?}, values.into_iter().map(|value| value.into()))"
            )
            .unwrap();
            output.push_str("    }\n");
            continue;
        }

        if parameter_names.len() >= 7 {
            output.push_str("    #[allow(clippy::too_many_arguments)]\n");
        }
        let parameters = parameter_names
            .iter()
            .map(|name| format!("{name}: impl Into<Value>"))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(output, "    pub fn {method_name}({parameters}) -> Self {{").unwrap();
        let arguments = parameter_names
            .iter()
            .map(|name| format!("{name}.into()"))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(output, "        Self::call({id:?}, [{arguments}])").unwrap();
        output.push_str("    }\n");
    }
    output.push_str("}\n\n");
}

fn method_name(type_name: &str, id: &str) -> String {
    if type_name == "Value" && id == "string" {
        "format_string".to_string()
    } else {
        identifier(id, "builtin")
    }
}

fn identifier(value: &str, fallback: &str) -> String {
    if value.starts_with('{') && value.ends_with('}') {
        return format!("arg_{}", &value[1..value.len() - 1]);
    }

    let mut result = String::new();
    let characters: Vec<char> = value.chars().collect();
    for (index, character) in characters.iter().enumerate() {
        if character.is_ascii_uppercase()
            && index > 0
            && !result.ends_with('_')
            && (characters[index - 1].is_ascii_lowercase()
                || characters
                    .get(index + 1)
                    .is_some_and(|next| next.is_ascii_lowercase()))
        {
            result.push('_');
        }
        if character.is_ascii_alphanumeric() {
            result.push(character.to_ascii_lowercase());
        } else if !result.ends_with('_') {
            result.push('_');
        }
    }
    while result.ends_with('_') {
        result.pop();
    }
    while result.starts_with('_') {
        result.remove(0);
    }
    if result.is_empty() {
        return fallback.to_string();
    }
    if is_keyword(&result) {
        result.push('_');
    }
    result
}

fn is_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}
