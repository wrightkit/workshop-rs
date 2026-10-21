use workshop_rs::actions::{ElementCountNode, ElementCountReport, ElementNodeKind};
use workshop_rs::parser;
use workshop_rs::{Action, Condition, Event, Program, Rule, Value, Variable};

use sha2::{Digest, Sha256};

use crate::common::{catalog, en, fixture, fixture_text};

const GLOBAL_ELEMENT_BUDGET: usize = 32768;

const PROBES: &[&str] = &[
    "control-minimal.ws",
    "arg-array-wide.ws",
    "arg-binder-lambda.ws",
    "arg-binder-source.ws",
    "arg-depth-deep.ws",
    "arg-line-bytes.ws",
    "arg-nodes-large.ws",
    "prophet-action-single.ws",
    "prophet-rule-flat.ws",
    "reproducer-bastion-prophet.ws",
    "rule-elements-large.ws",
    "rule-elements-split.ws",
    "rule-value-trees.ws",
];

#[test]
fn verbatim_probes_match_their_recorded_digests() {
    let expected: &[(&str, &str)] = &[
        (
            "reproducer-bastion-prophet.ws",
            "3a1cb89148242ca2ebe40d611ff033ea315b01d8246bde32b5f57c94b584c922",
        ),
        (
            "prophet-action-single.ws",
            "b14749af54e35a84a828dd22d887d968194b0f7af82eed47cf1575f2b6cdb04f",
        ),
    ];
    for &(name, sha256) in expected {
        let bytes = std::fs::read(fixture("import-limits", name)).unwrap();
        assert_eq!(format!("{:x}", Sha256::digest(&bytes)), sha256, "{name}");
    }
}

#[test]
fn the_vendored_bastion_build_anchors_the_accepted_side() {
    let text = fixture_text("real-projects", "bastion.ow");
    let program = parser::parse(&text, &catalog(), &en()).unwrap();
    let report = program.element_count(&catalog()).unwrap();

    assert_eq!(report.total, 29440);
    let rule_max = report.rules.iter().map(|rule| rule.count).max().unwrap();
    assert_eq!(rule_max, 1444);
    assert_eq!(max_action_count(&report), 606);
    let height = report
        .rules
        .iter()
        .map(ElementCountNode::height)
        .max()
        .unwrap();
    assert_eq!(height, 16);
}

fn report_for(name: &str) -> ElementCountReport {
    let text = fixture_text("import-limits", name);
    let program = parser::parse(&text, &catalog(), &en())
        .unwrap_or_else(|error| panic!("{name} must parse: {error}"));
    program
        .element_count(&catalog())
        .unwrap_or_else(|error| panic!("{name} must produce an element count: {error}"))
}

fn descendants<'a>(node: &'a ElementCountNode, out: &mut Vec<&'a ElementCountNode>) {
    for child in &node.children {
        out.push(child);
        descendants(child, out);
    }
}

fn max_action_count(report: &ElementCountReport) -> usize {
    let mut nodes = Vec::new();
    for rule in &report.rules {
        descendants(rule, &mut nodes);
    }
    nodes
        .iter()
        .filter(|node| node.kind == ElementNodeKind::Action)
        .map(|node| node.count)
        .max()
        .unwrap_or_default()
}

fn max_subtree_nodes(report: &ElementCountReport) -> usize {
    let mut nodes: Vec<&ElementCountNode> = report.rules.iter().collect();
    for rule in &report.rules {
        descendants(rule, &mut nodes);
    }
    nodes
        .iter()
        .map(|node| node.node_count())
        .max()
        .unwrap_or(1)
}

#[test]
fn probes_parse_and_stay_below_the_global_element_budget() {
    for name in PROBES {
        let report = report_for(name);
        assert!(
            report.total < GLOBAL_ELEMENT_BUDGET,
            "{name} must stay below the global budget: {}",
            report.total
        );
    }
}

#[test]
fn probe_metrics_match_the_recorded_structure() {
    let expected: &[(&str, usize, usize, usize, usize, usize)] = &[
        ("control-minimal.ws", 4, 4, 2, 5, 3),
        ("arg-array-wide.ws", 1004, 1004, 1002, 505, 4),
        ("arg-binder-lambda.ws", 17, 17, 15, 16, 8),
        ("arg-binder-source.ws", 14, 14, 12, 14, 7),
        ("arg-depth-deep.ws", 436, 436, 434, 245, 52),
        ("arg-line-bytes.ws", 164, 164, 162, 69, 5),
        ("arg-nodes-large.ws", 513, 513, 511, 515, 11),
        ("prophet-action-single.ws", 406, 406, 398, 323, 14),
        ("prophet-rule-flat.ws", 48, 48, 5, 91, 5),
        ("reproducer-bastion-prophet.ws", 3360, 3360, 422, 2507, 14),
        ("rule-elements-large.ws", 3522, 3522, 22, 1923, 4),
        ("rule-elements-split.ws", 3528, 882, 22, 483, 4),
        ("rule-value-trees.ws", 2490, 2490, 311, 2499, 11),
    ];
    for &(name, total, max_rule, max_action, max_nodes, height) in expected {
        let report = report_for(name);
        let rule_max = report.rules.iter().map(|rule| rule.count).max().unwrap();
        let tree_height = report
            .rules
            .iter()
            .map(ElementCountNode::height)
            .max()
            .unwrap();
        assert_eq!(
            (
                report.total,
                rule_max,
                max_action_count(&report),
                max_subtree_nodes(&report),
                tree_height
            ),
            (total, max_rule, max_action, max_nodes, height),
            "{name} structural metrics drifted; update probes or expectations deliberately"
        );
    }
}

#[test]
fn the_reproducer_keeps_the_rejected_rule_shape() {
    let report = report_for("reproducer-bastion-prophet.ws");
    assert_eq!(report.rules.len(), 1);
    let rule = &report.rules[0];

    let texts: Vec<_> = rule
        .children
        .iter()
        .filter(|node| node.name == "createInWorldText")
        .collect();
    assert_eq!(texts.len(), 8);
    assert_eq!(texts[0].count, 398);
    for (index, action) in texts.iter().enumerate() {
        assert_eq!(action.count, if index == 0 { 398 } else { 422 });
        assert_eq!(action.node_count(), 312);
        assert_eq!(action.statement_value_nodes(), 311);
        assert_eq!(action.height(), 13);
    }
    let flat = report_for("prophet-rule-flat.ws");
    assert_eq!(flat.rules[0].count, 48);
}

#[test]
fn height_and_node_count_measure_plain_subtree_shape() {
    let mut program = Program::default();
    program
        .global_variable(Variable::new("probe"))
        .rule(
            Rule::new("shape", Event::Global).action(Action::SetGlobalVariable {
                variable: "probe".to_string(),
                value: Value::call(
                    "add",
                    [
                        Value::number(1.0),
                        Value::call("add", [Value::number(2.0), Value::number(3.0)]),
                    ],
                ),
            }),
        );
    let report = program.element_count(&catalog()).unwrap();

    let rule = &report.rules[0];
    let action = &rule.children[0];
    let argument = &action.children[0];
    assert_eq!(argument.height(), 3);
    assert_eq!(argument.node_count(), 5);
    assert_eq!(action.height(), 4);
    assert_eq!(action.node_count(), 6);
    assert_eq!(rule.height(), 5);
    assert_eq!(rule.node_count(), 7);
}

#[test]
fn binder_probes_differ_only_in_scope_position() {
    fn call_name(value: &Value) -> &str {
        match value {
            Value::Call { name, .. } => name.as_str(),
            _ => panic!("probe statement carries a call chain: {value:?}"),
        }
    }
    let arg1_of_filter = |name: &str| {
        let text = fixture_text("import-limits", name);
        let program = parser::parse(&text, &catalog(), &en())
            .unwrap_or_else(|error| panic!("{name} must parse: {error}"));
        let Action::SetGlobalVariable { value, .. } = &program.rules[0].actions[0] else {
            panic!("{name}: one assignment statement");
        };
        let Value::Call { name: outer, args } = value else {
            panic!("{name}: the assigned value is a call")
        };
        assert_eq!(outer, "countOf");
        let Value::Call { name: filter, args } = &args[0] else {
            panic!("{name}: countOf's argument is the outer filter")
        };
        assert_eq!(filter, "filteredArray");
        (args[0].clone(), args[1].clone())
    };

    let (source, lambda) = arg1_of_filter("arg-binder-lambda.ws");
    assert_eq!(call_name(&source), "allPlayers");
    assert_eq!(
        call_name(&lambda),
        "isTrueForAll",
        "the nested binder sits in the per-element argument"
    );

    let (source, lambda) = arg1_of_filter("arg-binder-source.ws");
    assert_eq!(
        call_name(&source),
        "filteredArray",
        "the control chains the same binder through the source argument"
    );
    assert_eq!(call_name(&lambda), "!=");
}

#[test]
fn line_bytes_probe_exceeds_the_rejected_statement_size() {
    let longest_line = |name: &str| {
        fixture_text("import-limits", name)
            .lines()
            .map(str::len)
            .max()
            .unwrap_or_default()
    };
    assert!(
        longest_line("arg-line-bytes.ws") > longest_line("prophet-action-single.ws"),
        "the byte-axis probe must serialize a longer statement than the rejected verbatim one"
    );
}

#[test]
fn statement_value_nodes_isolate_a_statements_own_value_tree() {
    let big_tree = || {
        Value::call(
            "add",
            [
                Value::number(1.0),
                Value::call("add", [Value::number(2.0), Value::number(3.0)]),
            ],
        )
    };
    let mut program = Program::default();
    program.global_variable(Variable::new("probe")).rule(
        Rule::new("shape", Event::Global)
            .condition(Condition::new(big_tree()))
            .action(Action::If {
                condition: big_tree(),
            })
            .action(Action::SetGlobalVariable {
                variable: "probe".to_string(),
                value: big_tree(),
            })
            .action(Action::End),
    );
    let report = program.element_count(&catalog()).unwrap();
    let rule = &report.rules[0];
    let (condition, if_action) = (&rule.children[0], &rule.children[1]);
    let nested_action = &if_action.children[1];

    assert_eq!(condition.statement_value_nodes(), 5);
    assert_eq!(if_action.statement_value_nodes(), 5);
    assert_eq!(if_action.node_count(), 12);
    assert_eq!(nested_action.statement_value_nodes(), 5);
    assert_eq!(rule.statement_value_nodes(), 5);
    assert_eq!(rule.node_count(), 19);
}
