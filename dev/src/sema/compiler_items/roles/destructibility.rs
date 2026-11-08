//! Destruction role headers do not establish implementation or execution readiness.
use super::super::headers::{Flags, Trait};
use super::{
    Catalog, Declaration, Declarations, Error, LangItem, Report, TypeId, copying, item,
    validate_marker,
};
use crate::{
    resolve::model::DefId,
    support::constness::Constness,
    types::{Kind, arguments::Argument, callable::Convention},
};
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
    Err(error("invalid destruction role"))
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
                        | Kind::Adt(..)
                        | Kind::Tuple(_)
                        | Kind::Array { .. }
                        | Kind::ArrayTerm { .. }
                        | Kind::Slice(_)
                        | Kind::Pattern { .. }
                        | Kind::Closure { .. }
                )
            )))
}

pub(in crate::sema) fn require_source_drop_contract(
    report: &Report,
    facts: &Declarations,
) -> Result<(), Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    let Some(declaration) = catalog
        .validated_language(LangItem::Drop)
        .map_err(Error::unlocated)?
    else {
        return Err(Error::unlocated(
            "source destructor lost its actual Drop declaration",
        ));
    };
    let error = |message| Declarations::error(report, declaration.origin, message);
    let header = Trait::source(report, facts, declaration).map_err(error)?;
    let syntax = super::item(report, declaration)?;
    if !report
        .package
        .unit(declaration.origin.file)
        .configured
        .ast
        .list(syntax.where_clause)
        .is_empty()
    {
        return Err(error(
            "executable source Drop has unsupported defining conditions",
        ));
    }
    header
        .profile(
            &facts.types,
            Flags {
                unsafe_: false,
                constant: header.flags.constant,
                auto_: false,
            },
            0,
        )
        .map_err(error)?;
    if header.written.supertraits != 0 || header.members.is_empty() || header.members.len() > 2 {
        return Err(error(
            "executable source Drop requires its actual drop/pin_drop contract",
        ));
    }
    let mut drop = false;
    let mut pinned = false;
    for &member in header.members {
        let definition = report
            .resolution
            .definitions
            .get(member.0 as usize)
            .ok_or_else(|| error("Drop member lost its actual definition"))?;
        let pin = match report.package.names.get(definition.name) {
            "drop" if !drop => {
                drop = true;
                false
            }
            "pin_drop" if !pinned => {
                pinned = true;
                true
            }
            _ => {
                return Err(error(
                    "executable source Drop has another associated contract",
                ));
            }
        };
        require_method(report, facts, member, header.self_type, pin).map_err(error)?;
    }
    if !drop {
        return Err(error(
            "executable source Drop requires its actual drop member",
        ));
    }
    Ok(())
}

fn require_method(
    report: &Report,
    facts: &Declarations,
    member: DefId,
    subject: TypeId,
    pinned: bool,
) -> Result<(), &'static str> {
    let method = facts
        .function_header(member)?
        .ok_or("source Drop method header is pending")?;
    let source = method
        .declaration
        .source()
        .ok_or("Drop requires an actual source method")?;
    if report
        .resolution
        .definitions
        .get(member.0 as usize)
        .and_then(|d| d.node)
        != Some(source)
    {
        return Err("Drop method changed its actual source identity");
    }
    let ast = &report.package.unit(source.file).configured.ast;
    let crate::syntax::ast::Kind::Item(item) = ast.node(source.node).kind else {
        return Err("Drop method lost its actual declaration");
    };
    if !ast.list(ast.items[item as usize].where_clause).is_empty() {
        return Err("source Drop method has unsupported defining conditions");
    }
    let Some(Kind::Function {
        bound_regions: 1,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(method.signature)
    else {
        return Err(
            "source Drop requires a safe Rust method with one borrow binder and unit result",
        );
    };
    let [receiver] = facts
        .types
        .list(parameters)
        .ok_or("missing actual Drop parameters")?
    else {
        return Err("source Drop requires exactly its actual receiver");
    };
    if facts
        .generics
        .parameters
        .get(&member)
        .is_some_and(|p| !p.is_empty())
        || facts.is_const_function(member)
    {
        return Err("source Drop method has unsupported defining parameters");
    }
    let receiver = if pinned {
        pin_pointer(facts, *receiver)?
    } else {
        *receiver
    };
    if !matches!(facts.types.kind(receiver), Some(Kind::Reference {
        region, mutable: true, referent,
    }) if referent == subject && region.bound_parts() == Some((0, 0)))
    {
        return Err("source Drop method changed its actual unique Self borrow");
    }
    Ok(())
}

fn pin_pointer(facts: &Declarations, receiver: TypeId) -> Result<TypeId, &'static str> {
    let actual = facts
        .source_bindings
        .as_deref()
        .ok_or("missing source Pin catalog")?
        .language(LangItem::Pin)
        .ok_or("pinned Drop requires the actual Pin declaration")?;
    let Some(Kind::Adt(item, args)) = facts.types.kind(receiver) else {
        return Err("pinned Drop requires its actual Pin application");
    };
    let mut args = facts
        .types
        .arguments(args)
        .ok_or("invalid source Pin arguments")?;
    if item.0 != actual.definition.0 || args.len() != 1 {
        return Err("pinned Drop changed its actual Pin owner or argument arity");
    }
    let Some(Argument::Type(pointer)) = args.next() else {
        return Err("pinned Drop requires its actual pointer argument");
    };
    Ok(pointer)
}
