//! Typed source cfg literals select real owners and preserve opaque expression gaps.
use rustgm::{literal::Value, syntax::ast::Kind};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/literal-predicates";
const BOOLEAN: &str = "expected captured Boolean cfg literal";
fn reason(path: &str) -> Result<&'static str> {
    let file = path.rsplit('/').next().ok_or("fixture filename")?;
    Ok(match file {
        "wrong-string.rs" => "expected cfg predicate name",
        name if name.starts_with("wrong-") || name == "invalid-after-true.rs" => BOOLEAN,
        "boolean-as-string-value.rs" | "expression-group-string.rs" => {
            "expected captured string literal"
        }
        "literal-with-call.rs" | "literal-with-value.rs" => {
            "opaque cfg predicate has trailing tokens"
        }
        "literal-trailing-predicate.rs" => "cfg requires exactly one predicate",
        "expression-root-attribute.rs" | "expression-root-value.rs" | "expression-group.rs" => {
            "expected opaque metadata fragment"
        }
        "false-selection-opponent.rs" => "unresolved value name",
        _ => return Err("undeclared cfg opponent".into()),
    })
}
#[test]
fn source_boolean_literal_predicates_preserve_all_twenty_seven_outcomes() -> Result {
    let mut counts = [0; 2];
    let mut failures = Vec::new();
    for row in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, status) = row.split_once('\t').ok_or("cfg declaration row")?;
        let expected = status.parse::<usize>()?;
        let source = crate::resolve::fixture(path)?;
        let result = crate::resolve::source_provider(&source, "truth_captures");
        match (expected, result) {
            (0, Ok(_)) => {}
            (1, Err(error)) => {
                eprintln!("{path}: {error}");
                if !error.contains(reason(path)?) {
                    failures.push(format!("{path}: wrong rejection cause: {error}"));
                }
            }
            (0, Err(error)) => failures.push(format!("{path}: rejected: {error}")),
            (1, Ok(_)) => {
                failures.push(format!("{path}: accepted opponent or required profile gap"));
            }
            _ => return Err("unknown cfg source outcome".into()),
        }
        counts[expected] += 1;
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert_eq!(counts, [11, 16]);
    Ok(())
}
#[test]
fn captured_cfg_values_are_actual_typed_booleans_after_forwarding() -> Result {
    for (name, expected) in [
        ("value-true", true),
        ("value-false", false),
        ("forwarded-value", true),
    ] {
        let source = crate::resolve::fixture(&format!("{PACKET}/{name}.rs"))?;
        let report = crate::resolve::source_provider(&source, "truth_captures")?;
        let call = report.invocations.last().ok_or("cfg invocation")?;
        assert!(call.rule.is_none());
        assert_eq!(call.parser_calls, 0);
        let unit = report.package.unit(call.node.file);
        let Kind::Literal(index) = unit.configured.ast.node(call.node.node).kind else {
            return Err("cfg did not produce a typed literal".into());
        };
        assert!(
            matches!(unit.literal(index).map_err(|e| format!("{e:?}"))?, Value::Boolean(actual) if actual == expected)
        );
        assert_eq!(unit.source.original(), source);
    }
    Ok(())
}
