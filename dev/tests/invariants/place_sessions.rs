//! Node IDs and pending call offsets belong to one body, including new constants.
use super::*;
use crate::{
    resolve::unit::NodeKey,
    sema::{bodies::TypePrivacy, model::Function},
    types::TypeId,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture() -> Result<(
    crate::resolve::expansion::Report,
    crate::sema::model::Declarations,
    Function,
    NodeId,
)> {
    let source = crate::test_source::fixture("pa26/tests/spec/index-mut/shared.rs")?;
    let report = crate::test_source::resolve(&source, true)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let function = *facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "main"
        })
        .ok_or("main")?;
    let unit = &report.package.units[0].configured.ast;
    let node = unit
        .nodes
        .iter()
        .position(|node| matches!(node.kind, Kind::Index { .. }))
        .ok_or("actual index")?;
    let node = NodeId::new(u32::try_from(node)?);
    Ok((report, facts, function, node))
}
fn seed(checker: &mut Checker<'_>, function: Function, node: NodeId) -> Result {
    let term = checker
        .state
        .infer
        .known(&checker.facts.types, TypeId::UNIT)?;
    checker.state.file = function.node.file;
    let method = checker.retain_language_borrow(node, node, function.definition, term, term, term);
    checker.retain_language_index_call(node, Hook::Index, method, term)?;
    checker
        .state
        .language_places
        .as_mut()
        .ok_or("selected place")?
        .mutable
        .insert(node);
    assert_eq!(checker.language_index_method(node), Some(0));
    // Completed-body finalization releases signatures before the next entry.
    checker.state.calls.clear();
    Ok(())
}
#[test]
fn fresh_function_discards_previous_body_place_offsets_before_checking_the_same_node() -> Result {
    let (report, mut facts, function, node) = fixture()?;
    let mut checker = Checker::new(&report, &mut facts);
    seed(&mut checker, function, node)?;
    assert!(checker.start_function(function)?.is_none());
    let selected = checker.language_index_method(node).ok_or("new selection")?;
    assert!(selected < checker.state.call_types.methods.len());
    assert!(!checker.mutable_language_place(node));
    Ok(())
}
#[test]
fn fresh_constant_discards_previous_body_place_offsets_before_reusing_a_node() -> Result {
    let (report, mut facts, function, node) = fixture()?;
    let mut checker = Checker::new(&report, &mut facts);
    seed(&mut checker, function, node)?;
    checker.begin_constant(
        NodeKey {
            file: function.node.file,
            node,
        },
        TypeId::UNIT,
        report.resolution.scopes[0].module,
        TypePrivacy::Checked,
    );
    assert_eq!(checker.language_index_method(node), None);
    assert!(!checker.mutable_language_place(node));
    seed(&mut checker, function, node)?;
    assert_eq!(checker.state.call_types.methods.len(), 1);
    Ok(())
}
