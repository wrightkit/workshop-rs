use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use workshop_rs::catalog::{Catalog, CatalogEntry, Kind, Locale};
use workshop_rs::convert;
use workshop_rs::emitter;
use workshop_rs::parser;
use workshop_rs::roundtrip;
use workshop_rs::validate;

const EN_US: &str = "en-US";
const ZH_CN: &str = "zh-CN";
const SUPPORTED: &str = "✅ Supported";

struct InventoryRow {
    name: String,
    supported: bool,
}

#[test]
fn audited_catalog_features_have_individual_conformance_cases() {
    let catalog = Catalog::builtin().expect("builtin catalog");
    let mut failures = Vec::new();
    let mut summaries = BTreeMap::<&str, (usize, usize)>::new();

    for (category, kind, path) in [
        ("Actions", Kind::Action, "actions.md"),
        ("Values", Kind::Value, "values.md"),
        ("Events", Kind::Event, "events.md"),
        ("Operators", Kind::Operator, "operators.md"),
    ] {
        let failure_start = failures.len();
        let rows = inventory_rows(path);
        let mut documented = BTreeSet::new();
        for row in rows.iter().filter(|row| row.supported) {
            let case_id = format!("{category}/{}", slug(&row.name));
            let Some(entry) = catalog.resolve(kind, &Locale::new(EN_US), &row.name) else {
                if kind == Kind::Event && matches!(row.name.as_str(), "EventTeam" | "EventPlayer") {
                    continue;
                }
                failures.push(format!(
                    "{case_id}: audited spelling is absent from the catalog"
                ));
                continue;
            };
            documented.insert(entry.id.clone());
            let source = match kind {
                Kind::Action => call_program_source(&catalog, entry, false),
                Kind::Value => call_program_source(&catalog, entry, true),
                Kind::Event => event_source(entry),
                Kind::Operator => operator_source(&catalog, entry),
                _ => unreachable!(),
            };
            run_case(&case_id, &source, &catalog, &mut failures);
        }

        for entry in catalog.entries_of(kind) {
            if !documented.contains(&entry.id) {
                failures.push(format!(
                    "{category}/{}: catalog feature has no audited supported row",
                    entry.id
                ));
            }
        }
        summaries.insert(category, (documented.len(), failures.len() - failure_start));
    }

    let enum_rows = inventory_rows("enums.md");
    let enum_failure_start = failures.len();
    let mut enum_domains = BTreeSet::new();
    for row in enum_rows.iter().filter(|row| row.supported) {
        let case_id = format!("Constants/{}", slug(&row.name));
        let Some(domain) = catalog.enum_domain(&row.name) else {
            failures.push(format!(
                "{case_id}: audited enum domain is absent from the catalog"
            ));
            continue;
        };
        enum_domains.insert(domain.domain.clone());
        for member in &domain.members {
            let member_id = format!("{case_id}/{}", slug(&member.member));
            let source = enum_source(&catalog, &domain.domain, &member.member);
            run_enum_case(&member_id, &source, &catalog, &mut failures);
        }
    }
    for domain in catalog.enum_domains() {
        if !enum_domains.contains(&domain.domain) {
            failures.push(format!(
                "Constants/{}: catalog enum domain has no audited supported row",
                domain.domain
            ));
        }
    }
    summaries.insert(
        "Constants",
        (enum_domains.len(), failures.len() - enum_failure_start),
    );

    run_structural_cases(&catalog, &mut failures, &mut summaries);
    run_settings_cases(&catalog, &mut failures, &mut summaries);
    run_string_cases(&catalog, &mut failures, &mut summaries);

    let summary = summaries
        .iter()
        .map(|(category, (count, failure_count))| {
            format!("{category}: {count} cases, {failure_count} failures")
        })
        .collect::<Vec<_>>()
        .join("; ");
    eprintln!("Workshop conformance summary: {summary}");
    assert!(
        failures.is_empty(),
        "conformance failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn representative_invalid_inputs_preserve_diagnostic_boundaries() {
    let catalog = Catalog::builtin().expect("builtin catalog");
    let mut failures = Vec::new();

    let wrong_arity = program_source(
        "Set Invisible(All Players(All Teams), Color(White));",
        false,
    );
    expect_rejection(
        "invalid/category-wrong-enum-domain",
        &wrong_arity,
        &catalog,
        &mut failures,
    );

    let wrong_category = program_source(
        "Set Global Variable(probe, Create HUD Text(Event Player, 1, 1, 1, Left, 1, White, White, White, None, Default Visibility));",
        false,
    );
    expect_rejection(
        "invalid/category-action-in-value-position",
        &wrong_category,
        &catalog,
        &mut failures,
    );

    let wrong_arity = program_source("Wait();", false);
    expect_rejection("invalid/arity-wait", &wrong_arity, &catalog, &mut failures);

    assert!(
        failures.is_empty(),
        "negative conformance failures:\n{}",
        failures.join("\n")
    );
}

fn run_structural_cases(
    catalog: &Catalog,
    failures: &mut Vec<String>,
    summaries: &mut BTreeMap<&'static str, (usize, usize)>,
) {
    let failure_start = failures.len();
    let mut cases = Vec::new();
    for row in inventory_rows("structure.md")
        .into_iter()
        .filter(|row| row.supported)
    {
        let source = match row.name.as_str() {
            "settings" => settings_source(),
            "variables" => variables_source(),
            "subroutines" => subroutines_source(),
            "rule" => program_source("Wait(1, Ignore Condition);", false),
            "event" => event_source(catalog.entry(Kind::Event, "global").unwrap()),
            "conditions" => {
                program_with_conditions("If(True);\n    Wait(1, Ignore Condition);\nEnd;")
            }
            "actions" => program_source("Wait(1, Ignore Condition);", false),
            "disabled" => disabled_rule_source(),
            "global" | "Global Variable" => variables_source(),
            "Set Global Variable" | "Modify Global Variable" => variables_source(),
            "Set Global Variable At Index" | "Modify Global Variable At Index" => {
                variables_source()
            }
            "player" | "Player Variable" => player_variables_source(),
            "Set Player Variable" | "Modify Player Variable" => player_variables_source(),
            "Set Player Variable At Index" | "Modify Player Variable At Index" => {
                player_variables_source()
            }
            "Call Subroutine" | "Start Rule" | "Subroutine" => subroutines_source(),
            other => {
                failures.push(format!("Structure/{}: no source generator", slug(other)));
                continue;
            }
        };
        cases.push((format!("Structure/{}", slug(&row.name)), source));
    }

    for row in inventory_rows("control-flow.md")
        .into_iter()
        .filter(|row| row.supported)
    {
        let Some(action) = control_flow_action(&row.name) else {
            failures.push(format!(
                "Control Flow/{}: no source generator",
                slug(&row.name)
            ));
            continue;
        };
        cases.push((format!("Control Flow/{}", slug(&row.name)), action));
    }

    let count = cases.len();
    for (case_id, source) in cases {
        run_case(&case_id, &source, catalog, failures);
    }
    summaries.insert(
        "Structure and control flow",
        (count, failures.len() - failure_start),
    );
}

fn run_settings_cases(
    catalog: &Catalog,
    failures: &mut Vec<String>,
    summaries: &mut BTreeMap<&'static str, (usize, usize)>,
) {
    let failure_start = failures.len();
    let supported_sections = inventory_rows("settings.md")
        .into_iter()
        .filter(|row| row.supported)
        .map(|row| row.name)
        .collect::<BTreeSet<_>>();
    let count = supported_sections.len();
    run_case(
        "Settings/pixelart",
        include_str!("fixtures/settings/pixelart.settings.ws"),
        catalog,
        failures,
    );
    summaries.insert("Settings", (count, failures.len() - failure_start));
}

fn run_string_cases(
    catalog: &Catalog,
    failures: &mut Vec<String>,
    summaries: &mut BTreeMap<&'static str, (usize, usize)>,
) {
    let failure_start = failures.len();
    let supported = inventory_rows("strings.md")
        .into_iter()
        .filter(|row| row.supported)
        .map(|row| row.name)
        .collect::<BTreeSet<_>>();
    let mut count = 0;
    if supported.iter().any(|row| row.starts_with("Custom String")) {
        count += 1;
        run_case(
            "Strings/custom-string",
            &program_source(
                "Set Global Variable(probe, Custom String(\"probe {0}\", Global.probe));",
                false,
            ),
            catalog,
            failures,
        );
    } else {
        failures.push("Strings/custom-string: supported inventory row is missing".to_string());
    }

    let en_source = include_str!("fixtures/census/localization-en-us.ws");
    let zh_source = include_str!("fixtures/census/localization-zh-cn.ws");
    if supported.contains("en-US") {
        count += 1;
        run_case("Strings/en-us", en_source, catalog, failures);
    }
    if supported.contains("zh-CN") {
        count += 1;
        run_case_in_locale(
            "Strings/zh-cn",
            zh_source,
            &Locale::new(ZH_CN),
            catalog,
            failures,
        );
    }
    if supported
        .iter()
        .any(|row| row.starts_with("Bidirectional conversion"))
    {
        count += 1;
        let en = parser::parse_with_context(en_source, catalog, &Locale::new(EN_US), catalog)
            .expect("en-US localization fixture parses");
        let converted = convert::convert(
            en_source,
            catalog,
            &Locale::new(EN_US),
            &Locale::new(ZH_CN),
            &Default::default(),
        )
        .expect("en-US localization fixture converts");
        let zh = parser::parse_with_context(&converted.text, catalog, &Locale::new(ZH_CN), catalog)
            .expect("converted localization fixture parses");
        assert!(roundtrip::equivalent(&en, &zh));
    }
    summaries.insert(
        "Strings and localization",
        (count, failures.len() - failure_start),
    );
}

fn run_enum_case(case_id: &str, source: &str, catalog: &Catalog, failures: &mut Vec<String>) {
    let locale = Locale::new(EN_US);
    let program = match parser::parse_with_context(source, catalog, &locale, catalog) {
        Ok(program) => program,
        Err(error) => {
            failures.push(format!("{case_id}: parse failed: {error}"));
            return;
        }
    };
    if let Err(error) = validate::validate_canonical_ids(&program, catalog) {
        failures.push(format!("{case_id}: canonical validation failed: {error}"));
    } else if let Err(error) = program.validate() {
        failures.push(format!("{case_id}: semantic validation failed: {error}"));
    }
}

fn run_case(case_id: &str, source: &str, catalog: &Catalog, failures: &mut Vec<String>) {
    run_case_in_locale(case_id, source, &Locale::new(EN_US), catalog, failures);
}

fn run_case_in_locale(
    case_id: &str,
    source: &str,
    source_locale: &Locale,
    catalog: &Catalog,
    failures: &mut Vec<String>,
) {
    let target_locale = if source_locale.as_str() == EN_US {
        Locale::new(ZH_CN)
    } else {
        Locale::new(EN_US)
    };
    let program = match parser::parse_with_context(source, catalog, source_locale, catalog) {
        Ok(program) => program,
        Err(error) => {
            failures.push(format!("{case_id}: parse failed: {error}"));
            return;
        }
    };
    if let Err(error) = validate::validate_canonical_ids(&program, catalog) {
        failures.push(format!("{case_id}: canonical validation failed: {error}"));
        return;
    }
    if let Err(error) = program.validate() {
        failures.push(format!("{case_id}: semantic validation failed: {error}"));
        return;
    }
    let emitted = match emitter::emit(&program, catalog, source_locale) {
        Ok(text) => text,
        Err(error) => {
            failures.push(format!("{case_id}: emission failed: {error}"));
            return;
        }
    };
    let reparsed = match parser::parse_with_context(&emitted, catalog, source_locale, catalog) {
        Ok(program) => program,
        Err(error) => {
            failures.push(format!(
                "{case_id}: emitted output did not reparse: {error}"
            ));
            return;
        }
    };
    if let Err(error) = reparsed.validate() {
        failures.push(format!(
            "{case_id}: reparsed output did not validate: {error}"
        ));
        return;
    }
    if !roundtrip::equivalent(&program, &reparsed) {
        failures.push(format!(
            "{case_id}: emit -> parse changed canonical semantics"
        ));
        return;
    }
    let converted = match convert::convert(
        source,
        catalog,
        source_locale,
        &target_locale,
        &Default::default(),
    ) {
        Ok(output) => output,
        Err(error) => {
            failures.push(format!(
                "{case_id}: en-US -> zh-CN conversion failed: {error}"
            ));
            return;
        }
    };
    let converted_program =
        match parser::parse_with_context(&converted.text, catalog, &target_locale, catalog) {
            Ok(program) => program,
            Err(error) => {
                failures.push(format!(
                    "{case_id}: converted output did not parse: {error}"
                ));
                return;
            }
        };
    if let Err(error) = converted_program.validate() {
        failures.push(format!(
            "{case_id}: converted output did not validate: {error}"
        ));
    } else if !roundtrip::equivalent(&program, &converted_program) {
        failures.push(format!(
            "{case_id}: locale conversion changed canonical semantics"
        ));
    }
}

fn expect_rejection(case_id: &str, source: &str, catalog: &Catalog, failures: &mut Vec<String>) {
    let parsed = parser::parse_with_context(source, catalog, &Locale::new(EN_US), catalog);
    let rejected = match parsed {
        Err(_) => true,
        Ok(program) => {
            validate::validate_canonical_ids(&program, catalog).is_err()
                || program.validate().is_err()
        }
    };
    if !rejected {
        failures.push(format!("{case_id}: invalid input was accepted"));
    }
}

fn call_program_source(catalog: &Catalog, entry: &CatalogEntry, value: bool) -> String {
    let spelling = entry.spelling(&Locale::new(EN_US)).unwrap_or(&entry.id);
    if entry.id == "__forPlayerVariable__" {
        return program_source(
            "For Player Variable(Event Player, probe, 0, 1, 1);\n    Wait(1, Ignore Condition);\nEnd;",
            false,
        );
    }
    let args = (0..entry.params.len())
        .map(|index| sample_argument(catalog, entry, index))
        .collect::<Vec<_>>()
        .join(", ");
    let call = if entry.params.is_empty() {
        spelling.to_string()
    } else {
        format!("{spelling}({args})")
    };
    if value {
        call_program_source_text(&format!("Set Global Variable(probe, {call});"))
    } else {
        call_program_source_text(&format!("{call};"))
    }
}

fn event_source(entry: &CatalogEntry) -> String {
    let spelling = entry.spelling(&Locale::new(EN_US)).unwrap_or(&entry.id);
    let filters = if matches!(entry.id.as_str(), "global" | "subroutine") {
        String::new()
    } else {
        "        All;\n        All;\n".to_string()
    };
    let subroutine = if entry.id == "subroutine" {
        "        probe;\n"
    } else {
        ""
    };
    format!(
        "subroutines {{\n    0: probe\n}}\n\nrule (\"event\") {{\n    event {{\n        {spelling};\n{filters}{subroutine}    }}\n    actions {{\n        Wait(1, Ignore Condition);\n    }}\n}}\n"
    )
}

fn operator_source(_catalog: &Catalog, entry: &CatalogEntry) -> String {
    let spelling = entry.spelling(&Locale::new(EN_US)).unwrap_or(&entry.id);
    if matches!(entry.id.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=") {
        program_with_conditions(&format!(
            "If(1 {spelling} 1);\n    Wait(1, Ignore Condition);\nEnd;"
        ))
    } else {
        program_source(
            &format!("Modify Global Variable(probe, {spelling}, 1);"),
            false,
        )
    }
}

fn enum_source(catalog: &Catalog, domain: &str, member: &str) -> String {
    let domain_spelling = if domain == "Health" {
        domain
    } else {
        catalog
            .enum_domain(domain)
            .and_then(|entry| entry.spelling(&Locale::new(EN_US)))
            .unwrap_or(domain)
    };
    let member_spelling = catalog
        .enum_spelling(domain, &Locale::new(EN_US), member)
        .unwrap_or(member);
    if domain == "Health" {
        return program_source(
            &format!(
                "Set Global Variable(probe, Health Of Type(Event Player, {member_spelling}));"
            ),
            false,
        );
    }
    if domain == "Vector" {
        return program_source(
            &format!("Set Facing(Event Player, {member_spelling}, To World);"),
            false,
        );
    }
    if domain == "EventTeam" {
        return format!(
            "rule (\"event-filter\") {{\n    event {{\n        Ongoing - Each Player;\n        {member_spelling};\n        All;\n    }}\n    actions {{\n        Wait(1, Ignore Condition);\n    }}\n}}\n"
        );
    }
    if domain == "EventPlayer" {
        return format!(
            "rule (\"event-filter\") {{\n    event {{\n        Ongoing - Each Player;\n        All;\n        {member_spelling};\n    }}\n    actions {{\n        Wait(1, Ignore Condition);\n    }}\n}}\n"
        );
    }
    program_source(
        &format!("Set Global Variable(probe, {domain_spelling}({member_spelling}));"),
        false,
    )
}

fn sample_argument(catalog: &Catalog, entry: &CatalogEntry, index: usize) -> String {
    let parameter_type = entry.param_type(index).unwrap_or("Any");
    if let Some(domain) = entry.param_domain(index) {
        if parameter_type == "Any" || parameter_type == domain || parameter_type.contains(domain) {
            if let Some(enum_domain) = catalog.enum_domain(domain) {
                if let Some(member) = enum_domain.members.first() {
                    return catalog
                        .enum_spelling(domain, &Locale::new(EN_US), &member.member)
                        .unwrap_or(&member.member)
                        .to_string();
                }
            }
        }
    }
    if let Some(enum_domain) = catalog.enum_domain(parameter_type) {
        if parameter_type == "Vector" {
            return "Vector(0, 0, 0)".to_string();
        }
        if let Some(member) = enum_domain.members.first() {
            return catalog
                .enum_spelling(parameter_type, &Locale::new(EN_US), &member.member)
                .unwrap_or(&member.member)
                .to_string();
        }
    }
    match parameter_type {
        "Boolean" => "True".to_string(),
        "Number" | "Number|Boolean" | "Boolean|Number" | "Boolean|Number|Vector" => "1".to_string(),
        "Vector" | "Vector|Player" | "Vector|Player|Array" => "Vector(0, 0, 0)".to_string(),
        "String" | "String|Array" | "Object|String" => "Custom String(\"probe\")".to_string(),
        "Array" => "All Players(All Teams)".to_string(),
        "Player" | "Player|Array" | "Player|EntityId" => "Event Player".to_string(),
        "Object" | "Object|Array" | "Boolean|Number|Object|Array" => "Event Player".to_string(),
        "Hero|Array" => "D.Va".to_string(),
        "Operation" => "Add".to_string(),
        "Wait" => "Ignore Condition".to_string(),
        "__Operator__" => "==".to_string(),
        "Variable" | "Global Variable" | "Subroutine" => "probe".to_string(),
        "Player Variable" => "(Event Player).probe".to_string(),
        "EntityId" => value_spelling(catalog, "lastCreatedEntity", "Last Created Entity"),
        "TextId" => value_spelling(catalog, "lastTextId", "Last Text ID"),
        "AssistId" => value_spelling(catalog, "getLastAssistId", "Last Assist ID"),
        "DotId" => value_spelling(
            catalog,
            "getLastDamageOverTimeId",
            "Last Damage Over Time ID",
        ),
        "HotId" => value_spelling(
            catalog,
            "getLastHealingOverTimeId",
            "Last Heal Over Time ID",
        ),
        "HealthPoolId" => value_spelling(
            catalog,
            "getLastCreatedHealthPool",
            "Last Created Health Pool",
        ),
        "DamageModificationId" => value_spelling(
            catalog,
            "getLastDamageModification",
            "Last Damage Modification ID",
        ),
        "HealingModificationId" => value_spelling(
            catalog,
            "getLastHealingModification",
            "Last Healing Modification ID",
        ),
        _ => "1".to_string(),
    }
}

fn value_spelling(catalog: &Catalog, id: &str, fallback: &str) -> String {
    catalog
        .spelling(Kind::Value, &Locale::new(EN_US), id)
        .unwrap_or(fallback)
        .to_string()
}

fn call_program_source_text(action: &str) -> String {
    program_source(action, false)
}

fn program_source(action: &str, _unused: bool) -> String {
    format!(
        "variables {{\n    global:\n        0: probe\n    player:\n        0: probe\n}}\n\nsubroutines {{\n    0: probe\n}}\n\nrule (\"probe\") {{\n    event {{\n        Ongoing - Global;\n    }}\n    actions {{\n        {action}\n    }}\n}}\n"
    )
}

fn program_with_conditions(actions: &str) -> String {
    format!(
        "variables {{\n    global:\n        0: probe\n    player:\n        0: probe\n}}\n\nrule (\"probe\") {{\n    event {{\n        Ongoing - Global;\n    }}\n    conditions {{\n        True == True;\n    }}\n    actions {{\n        {actions}\n    }}\n}}\n"
    )
}

fn variables_source() -> String {
    program_source("Set Global Variable(probe, 1);", false)
}

fn player_variables_source() -> String {
    program_source("Set Player Variable(Event Player, probe, 1);", false)
}

fn subroutines_source() -> String {
    "subroutines {\n    0: probe\n}\n\nrule (\"probe\") {\n    event {\n        Subroutine;\n        probe;\n    }\n    actions {\n        Call Subroutine(probe);\n        Start Rule(probe, Restart Rule);\n    }\n}\n"
        .to_string()
}

fn disabled_rule_source() -> String {
    "disabled rule (\"probe\") {\n    event {\n        Ongoing - Global;\n    }\n    actions {\n        Wait(1, Ignore Condition);\n    }\n}\n"
        .to_string()
}

fn control_flow_action(name: &str) -> Option<String> {
    let action = match name {
        "If" => "If(True);\n    Wait(1, Ignore Condition);\nEnd;",
        "Else If" => {
            "If(True);\n    Wait(1, Ignore Condition);\nElse If(False);\n    Wait(1, Ignore Condition);\nEnd;"
        }
        "Else" => {
            "If(True);\n    Wait(1, Ignore Condition);\nElse;\n    Wait(1, Ignore Condition);\nEnd;"
        }
        "End" => "If(True);\n    Wait(1, Ignore Condition);\nEnd;",
        "While" => "While(True);\n    Wait(1, Ignore Condition);\nEnd;",
        "For Global Variable" => {
            "For Global Variable(probe, 0, 1, 1);\n    Wait(1, Ignore Condition);\nEnd;"
        }
        "For Player Variable" => {
            "For Player Variable(Event Player, probe, 0, 1, 1);\n    Wait(1, Ignore Condition);\nEnd;"
        }
        "Loop" => "Loop;",
        "Loop If" => "Loop If(True);",
        "Loop If Condition Is True" => "Loop If Condition Is True;",
        "Loop If Condition Is False" => "Loop If Condition Is False;",
        "Break" => "While(True);\n    Break;\nEnd;",
        "Continue" => "While(True);\n    Continue;\nEnd;",
        "Skip" => "Skip(1);",
        "Skip If" => "Skip If(True, 1);",
        "Wait" => "Wait(1, Ignore Condition);",
        "Wait Until" => "Wait Until(True, 1);",
        "Abort" => "Abort;",
        "Abort If" => "Abort If(True);",
        "Abort If Condition Is True" => "Abort If Condition Is True;",
        "Abort If Condition Is False" => "Abort If Condition Is False;",
        "Return" => "Return;",
        _ => return None,
    };
    Some(program_source(action, false))
}

fn settings_source() -> String {
    include_str!("fixtures/settings/pixelart.settings.ws").to_string()
}

fn inventory_rows(file: &str) -> Vec<InventoryRow> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/language-support");
    let text = std::fs::read_to_string(root.join(file)).expect("audited support inventory exists");
    text.lines()
        .filter_map(|line| {
            let cells = line
                .trim()
                .strip_prefix('|')?
                .strip_suffix('|')?
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>();
            if cells.len() < 2 || cells[0].eq_ignore_ascii_case("Feature") {
                return None;
            }
            let start = cells[0].find('`')? + 1;
            let end = cells[0][start..].find('`')? + start;
            let name = cells[0][start..end].to_string();
            Some(InventoryRow {
                name,
                supported: cells[1].contains(SUPPORTED),
            })
        })
        .collect()
}

fn slug(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
