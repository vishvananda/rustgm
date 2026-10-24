//! Actual builtin values retain ordinary call permissions and reject forged selectors.
use super::*;
use crate::{
    resolve::{expansion::Report, unit::NodeKey},
    sema::model::{Declarations, Operation},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-format-macros/literal-complete-{name}.rs"
    ))?)
}
fn expanded(name: &str) -> Result<Report> {
    Ok(crate::test_source::source_provider(
        &source(name)?,
        "formatting_calls",
    )?)
}
fn values(report: &Report) -> Result<Vec<NodeKey>> {
    let mut result = Vec::new();
    for (file, unit) in report.package.units.iter().enumerate() {
        for (node, actual) in unit.configured.ast.nodes.iter().enumerate() {
            if unit.configured.active[node]
                && matches!(
                    actual.kind,
                    Kind::LibraryValue {
                        kind: LibraryValue::FormatLiteral { .. },
                        ..
                    }
                )
            {
                result.push(NodeKey {
                    file: crate::resolve::unit::FileId(u32::try_from(file)?),
                    node: NodeId::new(u32::try_from(node)?),
                });
            }
        }
    }
    Ok(result)
}
fn checked_call(report: &Report, facts: &Declarations, key: NodeKey) -> Result {
    assert!(report.formatting_value(key)?.formatting());
    let selected = facts.methods.calls.get(&key).ok_or("actual checked call")?;
    let declaration = &report.resolution.definitions[selected.method.0 as usize];
    assert_eq!(report.package.names.get(declaration.name), "from_str");
    assert_eq!(selected.dereferences, 0);
    assert!(selected.borrow.is_none());
    assert!(
        matches!(facts.types.kind(selected.callable), Some(Type::FunctionItem { item, .. }) if item.0==selected.method.0)
    );
    let fact = facts.node(key).ok_or("actual result fact")?;
    let Some(Operation::Call {
        source_use: Some(use_),
        signature,
    }) = fact.operation
    else {
        return Err("ordinary source call context was not retained".into());
    };
    let owner = use_.owner(facts)?;
    assert_eq!(owner.file, key.file);
    let Type::Function {
        parameters, result, ..
    } = facts.types.kind(signature).ok_or("signature")?
    else {
        return Err("constructor is not an ordinary function call".into());
    };
    assert_eq!(facts.types.list(parameters).ok_or("inputs")?.len(), 1);
    assert_eq!(fact.ty, Some(result));
    let nominal = facts
        .source_bindings
        .as_deref()
        .ok_or("source")?
        .language(LangItem::FormatArguments)
        .ok_or("actual format_arguments")?
        .definition;
    assert!(matches!(facts.types.kind(result), Some(Type::Adt(item, _)) if item.0==nominal.0));
    Ok(())
}
#[test]
fn literal_values_publish_actual_callable_instances_and_current_body_effects() -> Result {
    for name in [
        "basic",
        "const",
        "newline",
        "empty",
        "escaped-braces",
        "raw",
        "renamed-role",
        "captured-expression",
    ] {
        let report = expanded(name)?;
        let facts = crate::pipeline::analyze_source(&report)?.facts;
        let keys = values(&report)?;
        assert_eq!(keys.len(), 1, "{name}");
        checked_call(&report, &facts, keys[0])?;
    }
    Ok(())
}
#[test]
fn canonical_format_result_does_not_authenticate_a_copied_tag_or_wrong_selector() -> Result {
    let mut report = expanded("basic")?;
    let key = values(&report)?[0];
    let unit = &mut report.package.units[key.file.0 as usize];
    let original = unit.configured.ast.node(key.node);
    let foreign = NodeId::new(u32::try_from(unit.configured.ast.nodes.len())?);
    unit.configured.ast.nodes.push(original);
    unit.configured.active.push(true);
    assert!(
        report
            .formatting_value(NodeKey {
                node: foreign,
                ..key
            })
            .is_err()
    );
    let unit = &mut report.package.units[key.file.0 as usize];
    let Kind::LibraryValue { value, .. } = unit.configured.ast.node(key.node).kind else {
        return Err("actual formatting result".into());
    };
    unit.configured.ast.nodes[key.node.get() as usize].kind = Kind::LibraryValue {
        kind: LibraryValue::FormatLiteral {
            constructor: NameId(u32::MAX),
        },
        value,
    };
    let error = crate::pipeline::analyze_source(&report)
        .err()
        .ok_or("forged selector accepted")?;
    assert!(
        error.to_string().contains("constructor selector"),
        "{error}"
    );
    Ok(())
}
#[test]
fn formatting_roles_keep_actual_lifetime_shape_and_unsafe_operation_checks() -> Result {
    let changed =
        source("basic")?.replace("struct TextArguments<'a>", "struct TextArguments<'a,'b>");
    let report = crate::test_source::source_provider(&changed, "wrong_format_owner")?;
    let error = crate::pipeline::analyze_source(&report)
        .err()
        .ok_or("wrong owner shape accepted")?;
    assert!(error.to_string().contains("one lifetime"), "{error}");
    let changed = source("basic")?.replace("pub const fn from_str", "pub const unsafe fn from_str");
    let report = crate::test_source::source_provider(&changed, "unsafe_format_constructor")?;
    let error = crate::pipeline::analyze_source(&report)
        .err()
        .ok_or("unsafe ctor gained lint permission")?;
    assert!(error.to_string().contains("unsafe context"), "{error}");
    Ok(())
}
#[test]
fn formatting_plan_is_cold_and_reports_containing_syntax_and_dependency_layouts() {
    let calls = super::super::calls::CallTypes::default();
    assert!(calls.formatting.is_none());
    eprintln!(
        "format layouts Plan={} Query={} CallTypes={} Requirement={} Dependency={} BodyWork={} SyntaxKind={} Node={} LibraryValue={} Catalog={}",
        size_of::<Pending>(),
        size_of::<PathQuery>(),
        size_of_val(&calls),
        size_of::<Requirement>(),
        size_of::<super::super::Dependency>(),
        size_of::<super::super::Work>(),
        size_of::<Kind>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<LibraryValue>(),
        size_of::<crate::sema::compiler_items::Catalog>()
    );
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    assert_eq!(size_of::<crate::body::ir::Value>(), 24);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
}
