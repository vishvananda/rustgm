//! Real closed helper selection keeps report ownership and registry readiness.
use super::*;
use crate::{pipeline, resolve::model::DefId, test_source, types::Primitive};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn helper_target_requires_actual_report_closed_arguments_and_registration() -> Result {
    let source = test_source::fixture("pa43/tests/spec/async-drop-helper/reified.rs")?;
    let report = test_source::source_provider(&source, "async_target")?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let definition = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.language(LangItem::AsyncDropInPlace))
        .ok_or("actual helper declaration")?
        .definition;
    let closed = facts
        .nodes
        .iter()
        .flatten()
        .filter_map(|node| node.ty)
        .find(|&ty| {
            matches!(facts.types.kind(ty), Some(Kind::FunctionItem { item, arguments, .. })
                if item.0 == definition.0 && facts.types.arguments(arguments).is_some_and(|mut args|
                    args.next() == Some(Argument::Type(TypeId::primitive(Primitive::U8)))
                    && args.next().is_none()))
        })
        .ok_or("actual reified closed helper")?;
    let selected = Instance::new(&facts.types, closed)?;
    let expected = Some(TypeId::primitive(Primitive::U8));
    for _ in 0..32 {
        assert_eq!(target(&report, &facts, selected)?, expected);
        assert!(crate::sema::compiler_items::check_const_drop_glue(&facts, selected).is_err());
    }
    let foreign = test_source::source_provider(&source, "other_async_target")?;
    assert!(target(&foreign, &facts, selected).is_err());
    facts
        .source_bindings
        .as_deref_mut()
        .ok_or("catalog")?
        .async_destruction_registration = Some(DefId(u32::MAX));
    assert!(target(&report, &facts, selected).is_err());
    facts
        .source_bindings
        .as_deref_mut()
        .ok_or("catalog")?
        .async_destruction_registration = None;
    assert_eq!(target(&report, &facts, selected)?, expected);
    let declared = facts.definitions[definition.0 as usize].ok_or("generic declaration")?;
    let open = Instance::new(&facts.types, declared)?;
    assert!(target(&report, &facts, open).is_err());
    Ok(())
}

#[test]
fn unused_static_receiver_cannot_authorize_general_async_cleanup_borrows() -> Result {
    let source = test_source::fixture("pa43/tests/spec/async-destructors/unused-static.rs")?;
    let report = test_source::source_provider(&source, "static_async_receiver")?;
    let facts = pipeline::analyze_source(&report)?.facts;
    let error = crate::sema::compiler_items::async_cleanup::operation(&report, &facts)
        .err()
        .ok_or("static receiver authorized a general cleanup borrow")?;
    assert!(
        error.message.contains("demanded AsyncDrop requires"),
        "{error}"
    );
    Ok(())
}
