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

/// Whether `predicate` is a validated `Destruct` trait obligation of any effect.
pub(crate) fn destruct_role(
    facts: &Declarations,
    predicate: crate::types::parameters::predicates::Predicate,
) -> Result<bool, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let Some((application, _)) = predicate.bound.trait_parts() else {
        return Ok(false);
    };
    let application = facts
        .types
        .trait_application(application)
        .ok_or("invalid Destruct trait application")?;
    let Some(declaration) = catalog.validated_role(LangItem::Destruct)? else {
        return Ok(false);
    };
    Ok(application.item.0 == declaration.definition.0
        && facts
            .types
            .arguments(application.arguments)
            .is_some_and(|args| args.len() == 0))
}

/// Called only for retained Builtin evidence after its exact formation was proved.
pub(crate) fn destruct_predicate(
    facts: &Declarations,
    predicate: crate::types::parameters::predicates::Predicate,
) -> Result<bool, &'static str> {
    if !destruct_role(facts, predicate)? {
        return Ok(false);
    }
    let Some((_, effect)) = predicate.bound.trait_parts() else {
        return Ok(false);
    };
    Ok(effect == Constness::Ordinary
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
        ))
}

pub(super) fn imported_drop(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    steps: &mut usize,
    complete: &mut super::RoleSet,
) -> Result<(), &'static str> {
    let Some(role) = catalog.role(LangItem::Drop)? else {
        return Ok(());
    };
    let header = Trait::current(report, facts, role, steps)?;
    profile(facts, &header)?;
    header.canonical_inputs(facts)?;
    let mut drop = false;
    let mut pinned = false;
    for &member in header.members {
        *steps = steps.saturating_add(6);
        let actual = facts
            .declaration_header(member)?
            .ok_or("imported Drop lost its actual method")?;
        if facts.associated.trait_owner(member) != Some(role.definition)
            || actual.declaration.source().is_some()
            || crate::sema::declaration_inputs::declared_own(
                &facts.generics,
                facts.crates.as_deref(),
                member,
            )?
            .iter()
            .next()
            .is_some()
        {
            return Err("imported Drop method changed its owner or defining conditions");
        }
        let is_pinned = method_kind(report, member, &mut drop, &mut pinned)?;
        typed_method(facts, member, actual.signature, header.self_type, is_pinned)?;
    }
    if !drop {
        return Err("imported Drop requires its actual drop member");
    }
    complete.insert(LangItem::Drop);
    Ok(())
}

pub(in crate::sema) fn require_source_drop_contract(
    report: &Report,
    facts: &Declarations,
) -> Result<(), Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    let Some(role) = catalog
        .validated_role(LangItem::Drop)
        .map_err(Error::unlocated)?
    else {
        return Err(Error::unlocated(
            "source destructor lost its actual Drop declaration",
        ));
    };
    let Some(declaration) = catalog.language(LangItem::Drop) else {
        // Imported headers already passed this role's typed checks.
        return if role.origin.source().is_none() {
            Ok(())
        } else {
            Err(Error::unlocated(
                "source destructor lost its actual Drop header",
            ))
        };
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
    profile(facts, &header).map_err(error)?;
    let mut drop = false;
    let mut pinned = false;
    for &member in header.members {
        let pin = method_kind(report, member, &mut drop, &mut pinned).map_err(error)?;
        require_method(report, facts, member, header.self_type, pin).map_err(error)?;
    }
    if !drop {
        return Err(error(
            "executable source Drop requires its actual drop member",
        ));
    }
    Ok(())
}

fn profile(facts: &Declarations, header: &Trait<'_>) -> Result<(), &'static str> {
    header.profile(
        &facts.types,
        Flags {
            unsafe_: false,
            constant: header.flags.constant,
            auto_: false,
        },
        0,
    )?;
    if header.written.supertraits != 0 || header.members.is_empty() || header.members.len() > 2 {
        return Err("executable source Drop requires its actual drop/pin_drop contract");
    }
    Ok(())
}

fn method_kind(
    report: &Report,
    member: DefId,
    drop: &mut bool,
    pinned: &mut bool,
) -> Result<bool, &'static str> {
    let definition = report
        .resolution
        .definitions
        .get(member.0 as usize)
        .ok_or("Drop member lost its actual definition")?;
    match report.package.names.get(definition.name) {
        "drop" if !std::mem::replace(drop, true) => Ok(false),
        "pin_drop" if !std::mem::replace(pinned, true) => Ok(true),
        _ => Err("executable source Drop has another associated contract"),
    }
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
    typed_method(facts, member, method.signature, subject, pinned)
}

fn typed_method(
    facts: &Declarations,
    member: DefId,
    signature: TypeId,
    subject: TypeId,
    pinned: bool,
) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions: 1,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
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

pub(in crate::sema) fn pin_pointer(
    facts: &Declarations,
    receiver: TypeId,
) -> Result<TypeId, &'static str> {
    let actual = facts
        .source_bindings
        .as_deref()
        .ok_or("missing source Pin catalog")?
        .role(LangItem::Pin)?
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
