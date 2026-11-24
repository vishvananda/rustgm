//! Defining-body context is authenticated separately from reached panic hooks.
use super::headers::{Flags, Trait};
use crate::{
    resolve::{
        bindings::{Declaration, Reservations},
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
        unit::NodeKey,
    },
    sema::{declaration_inputs, model::Declarations},
    support::{bindings::LangItem, constness::Constness, crates::generics::source::DiagnosticRole},
    syntax::{
        ast::Kind as Syntax,
        item::{Abi, ItemKind},
    },
    types::{
        GenericArgs, Kind, Primitive, RegionId, TypeId,
        arguments::Argument,
        callable::Convention,
        parameters::{ParameterKind, predicates::Bound},
    },
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
    let Some((role, declaration)) = LangItem::RUNTIME_FORMAT_BODIES
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
    for role in [LangItem::Panic].into_iter().chain(LangItem::FORMAT_PANICS) {
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
        role if tracks_caller(role) => {
            let body = header
                .body
                .and_then(crate::resolve::body_origin::BodyOrigin::source)
                .ok_or("source tracked panic requires its actual defined body")?;
            if !runtime_body(report, facts, definition, body)?
                || !facts
                    .checked_caller_tracking(report, definition)
                    .map_err(|_| "missing source formatting panic tracking")?
            {
                return Err(
                    "source formatting panic requires its pinned runtime and tracking policies",
                );
            }
        }
        LangItem::ConstPanicFmt
            if facts
                .const_panic_markers(report, definition)
                .map_err(|_| "missing source const_panic_fmt declaration policy")?
                != (false, true)
                || facts
                    .checked_caller_tracking(report, definition)
                    .map_err(|_| "missing source const_panic_fmt tracking")? =>
        {
            return Err("source const_panic_fmt requires its checked const body");
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
    if role == LangItem::PanicDisplay {
        let display = report
            .resolution
            .source_bindings
            .as_deref()
            .and_then(Reservations::display_declaration)
            .ok_or("source panic_display requires its actual Display diagnostic identity")?;
        let header = Trait::source(report, facts, display)?;
        display_scope(facts, definition, &header)?;
    } else if LangItem::FORMAT_PANICS.contains(&role) {
        empty_input_scope(facts, definition)?;
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
    let candidates = LangItem::RUNTIME_FORMAT_BODIES
        .into_iter()
        .map(|role| ((), role));
    let Some(((), role)) = catalog.matching_role(definition, candidates)? else {
        return Ok(false);
    };
    catalog.validated_role(role)?;
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
    if !LangItem::ARGUMENTS_PANICS.contains(&role) {
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

fn tracks_caller(role: LangItem) -> bool {
    LangItem::RUNTIME_FORMAT_BODIES.contains(&role)
}

fn empty_input_scope(facts: &Declarations, definition: DefId) -> Result<(), &'static str> {
    if declaration_inputs::declared_own(&facts.generics, facts.crates.as_deref(), definition)?.len()
        != 0
    {
        return Err("source panic declaration introduces a defining condition");
    }
    Ok(())
}

fn display_scope(
    facts: &Declarations,
    definition: DefId,
    header: &Trait<'_>,
) -> Result<(), &'static str> {
    const BOUND: &str = "source panic_display requires its exact ordinary T: Display bound";
    header.profile(
        &facts.types,
        Flags {
            unsafe_: false,
            constant: false,
            auto_: false,
        },
        0,
    )?;
    let [generic] = facts
        .generics
        .parameters
        .get(&definition)
        .map_or(&[][..], Vec::as_slice)
    else {
        return Err(BOUND);
    };
    let Argument::Type(ty) = generic.argument else {
        return Err(BOUND);
    };
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source panic_display has no actual provider catalog")?;
    let meta = catalog
        .role(LangItem::MetaSized)?
        .map(|role| role.definition.0);
    let inputs =
        declaration_inputs::declared_own(&facts.generics, facts.crates.as_deref(), definition)?;
    if inputs.len() > 3 {
        return Err(BOUND);
    }
    let mut found = false;
    for input in inputs.iter() {
        let predicate = input.value.ordinary()?;
        if predicate.ty != ty {
            return Err(BOUND);
        }
        if predicate.bound == Bound::Sized {
            continue;
        }
        let Some((application, Constness::Ordinary)) = predicate.bound.trait_parts() else {
            return Err(BOUND);
        };
        let application = facts.types.trait_application(application).ok_or(BOUND)?;
        if application.arguments != GenericArgs::EMPTY {
            return Err(BOUND);
        }
        if application.item.0 == header.definition.0 && !found {
            found = true;
        } else if meta != Some(application.item.0) {
            return Err(BOUND);
        }
    }
    if !found {
        return Err(BOUND);
    }
    Ok(())
}

/// A consumer binds an imported panic role only after re-proving the installed
/// declaration it names; the provider's completion is never trusted.
pub(super) fn imported(
    report: &Report,
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
        if role == LangItem::PanicDisplay {
            let display = facts
                .source_bindings
                .as_deref()
                .ok_or("imported panic_display lost its actual provider catalog")?
                .imported_diagnostic(DiagnosticRole::Display, DefKind::Trait, steps)?
                .ok_or("imported panic_display requires its actual Display diagnostic identity")?;
            let bound = Trait::current(report, facts, display, steps)?;
            *steps = steps.saturating_add(4);
            display_scope(facts, definition, &bound)?;
        } else {
            empty_input_scope(facts, definition)?;
        }
        if facts.recorded_caller_tracking(definition)? != tracks_caller(role)
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
        LangItem::PanicFmt | LangItem::ConstPanicFmt => {
            let declaration = format_arguments
                .ok_or("source formatting panic requires actual format_arguments identity")?;
            if bound_regions != 1
                || !parameters.is_empty()
                || !matches!(facts.types.kind(*parameter),
                Some(Kind::Adt(item, arguments)) if item.0 == declaration.0
                    && facts.types.arguments(arguments).is_some_and(|mut args|
                        matches!(args.next(), Some(Argument::Lifetime(region))
                            if region.bound_parts() == Some((0, 0))) && args.next().is_none()))
            {
                return Err("source formatting panic changed its real Arguments signature");
            }
        }
        LangItem::PanicDisplay => {
            let [generic] = parameters else {
                return Err("source panic_display requires its actual T parameter");
            };
            let Argument::Type(ty) = generic.argument else {
                return Err("source panic_display requires its actual type parameter");
            };
            if bound_regions != 1
                || !matches!(facts.types.kind(*parameter),
                    Some(Kind::Reference { referent, region, mutable: false })
                        if ty == referent && region.bound_parts() == Some((0, 0)))
                || !matches!(facts.types.kind(ty), Some(Kind::Parameter(id))
                    if facts.types.parameter(id).is_some_and(|slot|
                        slot.owner.0 == definition.0 && slot.index == 0
                            && slot.kind == ParameterKind::Type { sized: true }))
            {
                return Err("source panic_display requires its genuine shared T argument");
            }
        }
        _ => return Err("unsupported source panic defining-context role"),
    }
    Ok(())
}
