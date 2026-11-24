//! Defining-body context is authenticated separately from reached panic hooks.
use crate::{
    resolve::{
        bindings::Declaration, expansion::Report, model::DefId, package::Error, unit::NodeKey,
    },
    sema::model::Declarations,
    support::bindings::LangItem,
    syntax::{
        ast::Kind as Syntax,
        item::{Abi, ItemKind},
    },
    types::{Kind, Primitive, RegionId, TypeId, arguments::Argument, callable::Convention},
};

pub(in crate::sema) fn runtime_body(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
    body: NodeKey,
) -> Result<bool, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let Some((role, declaration)) = [LangItem::PanicFmt, LangItem::PanicDisplay]
        .into_iter()
        .find_map(|role| {
            catalog
                .language(role)
                .filter(|declaration| declaration.definition == definition)
                .map(|declaration| (role, declaration))
        })
    else {
        return Ok(false);
    };
    crate::sema::header_attributes::charge_source_contract_query(facts, 8)
        .map_err(|_| "source panic defining-context work budget exceeded")?;
    catalog.require_provider_report(report)?;
    let function = facts
        .function_indices
        .get(&definition)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("source panic body requires its completed actual function")?;
    if declaration.parent.is_some()
        || function.definition != definition
        || function.node != declaration.origin
        || function.body != Some(body)
    {
        return Err("source panic defining body changed its actual role/function identity");
    }
    let arguments = catalog
        .language(LangItem::FormatArguments)
        .map(|declaration| declaration.definition);
    signature(facts, definition, function.signature, role, arguments)?;
    let unit = report.package.unit(declaration.origin.file);
    if !matches!(unit.configured.ast.node(declaration.origin.node).kind,
        Syntax::Item(index) if matches!(unit.configured.ast.items[index as usize].kind,
            ItemKind::Function { flags, abi: Abi::Rust, body: Some(actual), .. }
                if flags.constant && flags.safety == crate::syntax::item::Safety::Default
                    && !flags.asynchronous && actual == body.node))
    {
        return Err("source panic defining context requires its current const Rust body");
    }
    if facts
        .const_panic_markers(report, definition)
        .map_err(|_| "source panic body lost its current completed header")?
        != (true, true)
    {
        return Err("source panic body requires both original checked const-policy markers");
    }
    Ok(true)
}

/// A conforming local declaration completes its actual role for export.
pub(in crate::sema) fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    for role in [LangItem::Panic, LangItem::PanicFmt, LangItem::PanicNounwind] {
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(());
        };
        let Some(declaration) = catalog.language(role) else {
            continue;
        };
        crate::sema::header_attributes::charge_source_contract_query(facts, 4)?;
        catalog
            .require_provider_report(report)
            .map_err(|message| Declarations::error(report, declaration.origin, message))?;
        if conforming(report, facts, declaration, role).is_ok() {
            super::roles::complete_panic_role(facts, role, declaration)?;
        }
    }
    Ok(())
}

fn conforming(
    report: &Report,
    facts: &Declarations,
    declaration: Declaration,
    role: LangItem,
) -> Result<(), &'static str> {
    let header = super::headers::functions::current(facts, declaration)?;
    let definition = declaration.definition;
    match role {
        LangItem::PanicFmt => {
            let body = header
                .body
                .and_then(crate::resolve::body_origin::BodyOrigin::source)
                .ok_or("source panic_fmt requires its actual defined body")?;
            if !runtime_body(report, facts, definition, body)?
                || !facts
                    .checked_caller_tracking(report, definition)
                    .map_err(|_| "missing source panic_fmt tracking")?
            {
                return Err("source panic_fmt requires its pinned runtime and tracking policies");
            }
        }
        LangItem::PanicNounwind
            if !facts
                .source_nounwind_marker(report, definition)
                .map_err(|_| "missing source nounwind policy")?
                || facts
                    .const_panic_markers(report, definition)
                    .map_err(|_| "missing source panic_nounwind const policy")?
                    != (false, true)
                || facts
                    .checked_caller_tracking(report, definition)
                    .map_err(|_| "missing source panic_nounwind tracking")? =>
        {
            return Err("source panic_nounwind requires its untracked non-unwinding declaration");
        }
        _ => {}
    }
    if LangItem::FORMAT_PANICS.contains(&role) {
        input_scope(facts, definition)?;
    }
    signature(
        facts,
        definition,
        header.signature,
        role,
        format_arguments(facts, role)?,
    )
}

pub(crate) fn runtime_format_entry(
    facts: &Declarations,
    definition: DefId,
) -> Result<bool, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    if catalog
        .role(LangItem::PanicFmt)?
        .is_none_or(|role| role.definition != definition)
    {
        return Ok(false);
    }
    catalog.validated_role(LangItem::PanicFmt)?;
    Ok(true)
}

pub(in crate::sema) fn runtime_panic_declaration(
    facts: &Declarations,
    definition: DefId,
) -> Result<bool, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    for role in LangItem::FORMAT_PANICS {
        if catalog
            .role(role)?
            .is_some_and(|declaration| declaration.definition == definition)
        {
            catalog.validated_role(role)?;
            return Ok(true);
        }
    }
    Ok(false)
}

fn format_arguments(facts: &Declarations, role: LangItem) -> Result<Option<DefId>, &'static str> {
    if role != LangItem::PanicFmt {
        return Ok(None);
    }
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source panic role has no catalog")?;
    Ok(catalog
        .role(LangItem::FormatArguments)?
        .map(|declaration| declaration.definition))
}

fn input_scope(facts: &Declarations, definition: DefId) -> Result<(), &'static str> {
    if crate::sema::declaration_inputs::declared_own(
        &facts.generics,
        facts.crates.as_deref(),
        definition,
    )?
    .len()
        != 0
    {
        return Err("source panic declaration introduces a defining condition");
    }
    Ok(())
}

/// A consumer binds an imported panic role only after re-proving the installed
/// declaration it names; the provider's completion is never trusted.
pub(super) fn imported(
    facts: &Declarations,
    declaration: super::Role,
    role: LangItem,
    steps: &mut usize,
) -> Result<(), &'static str> {
    *steps = steps.saturating_add(4);
    facts
        .associated
        .imported_owner()
        .ok_or("imported panic role lost its actual owner/schema graph")?
        .declared_panic_role(facts, declaration, role, steps)?;
    let definition = declaration.definition;
    if declaration.parent.is_some() || facts.associated.owner(definition).is_some() {
        return Err("imported panic role requires its actual free declaration");
    }
    let header = facts
        .declaration_header(definition)?
        .ok_or("imported panic role lost its installed declaration header")?;
    if header.declaration != declaration.origin || header.body.is_none() {
        return Err("imported panic role changed its declaration or body identity");
    }
    if LangItem::FORMAT_PANICS.contains(&role) {
        input_scope(facts, definition)?;
        if facts.recorded_caller_tracking(definition)? != (role == LangItem::PanicFmt)
            || role == LangItem::PanicNounwind
                && !crate::resolve::package::Package::target_aborts_panics()
        {
            return Err("imported panic role changed its defining caller or unwind policy");
        }
    }
    if role == LangItem::Panic || LangItem::FORMAT_PANICS.contains(&role) {
        signature(
            facts,
            definition,
            header.signature,
            role,
            format_arguments(facts, role)?,
        )?;
    }
    Ok(())
}

fn signature(
    facts: &Declarations,
    definition: DefId,
    callable: TypeId,
    role: LangItem,
    format_arguments: Option<DefId>,
) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
        ..
    }) = facts.types.kind(callable)
    else {
        return Err("source panic role requires its safe Rust function signature");
    };
    let Some([parameter]) = facts.types.list(parameters) else {
        return Err("source panic role requires its actual single argument");
    };
    if facts.types.kind(result) != Some(Kind::Never) || !facts.is_const_function(definition) {
        return Err("source panic role requires a const declaration with never result");
    }
    let parameters = facts
        .generics
        .parameters
        .get(&definition)
        .map_or(&[][..], Vec::as_slice);
    match role {
        LangItem::Panic | LangItem::PanicNounwind => {
            if role == LangItem::PanicNounwind && bound_regions != 0
                || !parameters.is_empty()
                || !matches!(facts.types.kind(*parameter),
                Some(Kind::Reference { referent, region, mutable: false })
                    if referent == TypeId::primitive(Primitive::Str) && region == RegionId::STATIC)
            {
                return Err("source panic changed its real static str signature");
            }
        }
        LangItem::PanicFmt => {
            let declaration = format_arguments
                .ok_or("source panic_fmt requires actual format_arguments identity")?;
            if bound_regions != 1
                || !parameters.is_empty()
                || !matches!(facts.types.kind(*parameter),
                Some(Kind::Adt(item, arguments)) if item.0 == declaration.0
                    && facts.types.arguments(arguments).is_some_and(|mut args|
                        matches!(args.next(), Some(Argument::Lifetime(region))
                            if region.bound_parts() == Some((0, 0))) && args.next().is_none()))
            {
                return Err("source panic_fmt changed its real Arguments signature");
            }
        }
        LangItem::PanicDisplay => {
            let [generic] = parameters else {
                return Err("source panic_display requires its actual T parameter");
            };
            if !matches!((generic.argument, facts.types.kind(*parameter)),
                (Argument::Type(ty), Some(Kind::Reference { referent, mutable: false, .. })) if ty == referent)
            {
                return Err("source panic_display requires its genuine shared T argument");
            }
        }
        _ => return Err("unsupported source panic defining-context role"),
    }
    Ok(())
}
