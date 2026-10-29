//! The genuine cold hint is admitted with its ordinary, empty source fallback.
use crate::{
    resolve::expansion::Report,
    sema::model::{Declarations, Function},
    syntax::{ast::Kind as Syntax, item::ItemKind},
    types::{Kind, TypeId, callable::Convention},
};
pub(super) fn validate(
    report: &Report,
    facts: &Declarations,
    function: Function,
) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err("source cold_path requires its safe Rust ()->() signature");
    };
    if facts.types.list(parameters).is_none_or(|p| !p.is_empty())
        || !facts.is_const_function(function.definition)
        || facts
            .generics
            .parameters
            .get(&function.definition)
            .is_some_and(|p| !p.is_empty())
        || facts
            .generics
            .declared_predicates_in_scope(function.definition)
            .next()
            .is_some()
        || facts
            .lifetimes
            .functions
            .get(&function.definition)
            .is_some_and(|l| l.total != 0 || l.captures != 0)
    {
        return Err(
            "source cold_path requires its actual const fallback without parameters or predicates",
        );
    }
    let unit = report.package.unit(function.node.file);
    let Syntax::Item(index) = unit.configured.ast.node(function.node.node).kind else {
        return Err("source cold_path lost its actual item");
    };
    let ItemKind::Function {
        body: Some(body), ..
    } = unit.configured.ast.items[index as usize].kind
    else {
        return Err("source cold_path requires its genuine source fallback body");
    };
    if function.body
        != Some(crate::resolve::unit::NodeKey {
            file: function.node.file,
            node: body,
        })
        || !matches!(unit.configured.ast.node(body).kind, Syntax::Block { statements, tail: None, .. } if unit.configured.ast.list(statements).is_empty())
    {
        return Err("source cold_path requires its genuine empty source fallback");
    }
    if !crate::sema::header_attributes::intrinsic_fallback_is_spec(
        facts,
        report,
        function.definition,
    )
    .map_err(|_| "source cold_path has no authenticated fallback metadata")?
    {
        return Err("source cold_path requires actual miri::intrinsic_fallback_is_spec metadata");
    }
    Ok(())
}
