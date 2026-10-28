//! Destruction role headers do not establish implementation or execution readiness.
use super::{
    Catalog, Declaration, Declarations, DefKind, Error, ItemKind, Kind, LangItem, Report, TypeId,
    copying, function, item, validate_marker,
};
use crate::{support::constness::Constness, types::callable::Convention};
#[cfg(test)]
#[path = "../../../../tests/invariants/registration_seal.rs"]
mod tests;
pub(super) fn validate(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    role: LangItem,
    self_type: TypeId,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let syntax = item(report, declaration)?;
    let ast = &report.package.unit(declaration.origin.file).configured.ast;
    if !ast.list(syntax.where_clause).is_empty() {
        return Err(error(
            "source destruction role has unsupported defining constraints",
        ));
    }
    if role == LangItem::Destruct {
        copying::require_super(
            report,
            facts,
            catalog,
            declaration,
            self_type,
            LangItem::PointeeSized,
            Constness::Ordinary,
        )?;
        return validate_marker(report, facts, declaration);
    }
    if role != LangItem::Drop {
        return Err(Declarations::error(
            report,
            declaration.origin,
            "invalid destruction role",
        ));
    }
    let ItemKind::Trait { bounds, .. } = syntax.kind else {
        return Err(error("source Drop requires its actual trait declaration"));
    };
    if !ast.list(bounds).is_empty() {
        return Err(error("source Drop has unsupported defining constraints"));
    }
    let members = facts
        .associated
        .owner(declaration.definition)
        .and_then(|owner| facts.associated.owner_members(owner))
        .ok_or_else(|| error("source Drop has no completed member inventory"))?;
    let [method] = members else {
        return Err(error("full source Drop/Pin contracts are not implemented"));
    };
    let method_definition = &report.resolution.definitions[method.0 as usize];
    if method_definition.kind != DefKind::Function
        || report.package.names.get(method_definition.name) != "drop"
    {
        return Err(error("source Drop requires its actual drop member"));
    }
    let method_node = method_definition
        .node
        .ok_or_else(|| error("source Drop member lost its declaration"))?;
    let method_ast = &report.package.unit(method_node.file).configured.ast;
    let crate::syntax::ast::Kind::Item(method_item) = method_ast.node(method_node.node).kind else {
        return Err(error("source Drop member lost its declaration"));
    };
    let method_syntax = method_ast.items[method_item as usize];
    if !method_ast.list(method_syntax.where_clause).is_empty() {
        return Err(error(
            "source Drop member has unsupported defining constraints",
        ));
    }
    let function = function(facts, *method)
        .ok_or_else(|| error("source Drop member signature is not prepared"))?;
    let Some(Kind::Function {
        bound_regions: 1,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err(error(
            "source Drop requires a safe Rust method with one borrow binder",
        ));
    };
    let parameters = facts
        .types
        .list(parameters)
        .ok_or_else(|| error("source Drop member inputs are missing"))?;
    let [receiver] = parameters else {
        return Err(error("source Drop requires exactly one unique receiver"));
    };
    if result != TypeId::UNIT
        || !matches!(facts.types.kind(*receiver), Some(Kind::Reference {
            mutable: true, referent, region,
        }) if referent == self_type && region.bound_parts() == Some((0, 0)))
        || facts
            .generics
            .parameters
            .get(method)
            .is_some_and(|p| !p.is_empty())
        || facts.is_const_function(*method)
    {
        return Err(error(
            "source Drop member must retain the exact &mut Self to unit contract",
        ));
    }
    Ok(())
}

/// Called only for retained Builtin evidence after its exact formation was proved.
pub(crate) fn destruct_predicate(
    facts: &Declarations,
    predicate: crate::types::parameters::predicates::Predicate,
) -> Result<bool, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let Some((application, effect)) = predicate.bound.trait_parts() else {
        return Ok(false);
    };
    let application = facts
        .types
        .trait_application(application)
        .ok_or("invalid Destruct trait application")?;
    let Some(declaration) = catalog.validated_language(LangItem::Destruct)? else {
        return Ok(false);
    };
    Ok(application.item.0 == declaration.definition.0
        && facts
            .types
            .arguments(application.arguments)
            .is_some_and(|args| args.len() == 0)
        && (effect == Constness::Ordinary
            || matches!(
                facts.types.kind(predicate.ty),
                Some(
                    Kind::Primitive(_)
                        | Kind::Never
                        | Kind::Pointer { .. }
                        | Kind::Reference { .. }
                        | Kind::Function { .. }
                        | Kind::FunctionItem { .. }
                )
            )))
}
