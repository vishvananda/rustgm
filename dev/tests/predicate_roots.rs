//! Source macro roots preserve commas belonging to a nested predicate.
use rustgm::{literal::Value, syntax::ast::Kind};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/source-cfg-root-commas";

fn observed(source: &str, expected: bool) -> Result {
    let report = crate::resolve::source_provider(source, "predicate_roots")?;
    let call = report
        .invocations
        .last()
        .ok_or("missing actual cfg invocation")?;
    assert!(call.rule.is_none());
    let unit = report.package.unit(call.node.file);
    let Kind::Literal(index) = unit.configured.ast.node(call.node.node).kind else {
        return Err("cfg did not construct a typed literal".into());
    };
    let literal = unit.literal(index).map_err(|error| format!("{error:?}"))?;
    assert!(matches!(literal,Value::Boolean(value) if value==expected));
    assert_eq!(unit.source.original(), source);
    Ok(())
}

#[test]
fn source_cfg_preserves_nested_commas_and_one_optional_macro_comma() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(PACKET);
    let table = std::fs::read_to_string(root.join("cases.tsv"))?;
    let mut count = 0;
    for row in table.lines() {
        let fields: Vec<_> = row.split('\t').collect();
        let [name, status, boolean, message] = fields.as_slice() else {
            return Err("invalid cfg-root contract".into());
        };
        let source = std::fs::read_to_string(root.join(format!("{name}.rs")))?;
        if *status == "0" {
            observed(&source, boolean.parse()?)?;
        } else {
            let error = crate::resolve::source_provider(&source, "predicate_roots")
                .err()
                .ok_or("invalid source cfg predicate accepted")?;
            assert!(error.contains(message), "{name}: {error}");
            assert!(
                !error.contains("budget") && !error.contains("not implemented"),
                "{error}"
            );
        }
        count += 1;
    }
    assert_eq!(count, 19);
    let ordinary = std::fs::read_to_string(root.join("ordinary.rs"))?;
    let error = crate::resolve::resolve(&ordinary, true)
        .err()
        .ok_or("ordinary cfg profile changed")?;
    assert!(
        error.contains("expected cfg predicate arguments"),
        "{error}"
    );
    let foundation = std::fs::read_to_string(root.join("foundation.rs"))?;
    for width in [8, 32, 128] {
        let source = format!(
            "{foundation}pub const OBSERVER:bool=cfg!(all({}));",
            "unix,".repeat(width)
        );
        observed(&source, true)?;
    }
    Ok(())
}
