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

/// A conforming source `panic` declaration completes its role. Any other form
/// stays pending: local checking is unchanged and export rejects it by name.
/// `panic_fmt` waits for its lifetime-parameterized `Arguments` to transport.
pub(in crate::sema) fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(declaration) = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.language(LangItem::Panic))
    else {
        return Ok(());
    };
    crate::sema::header_attributes::charge_source_contract_query(facts, 4)?;
    facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| Error::unlocated("source panic role catalog disappeared"))?
        .require_provider_report(report)
        .map_err(|message| Declarations::error(report, declaration.origin, message))?;
    if conforming(facts, declaration).is_ok() {
        super::roles::complete_panic_role(facts, LangItem::Panic, declaration)?;
    }
    Ok(())
}

fn conforming(facts: &Declarations, declaration: Declaration) -> Result<(), &'static str> {
    let header = super::headers::functions::current(facts, declaration)?;
    signature(
        facts,
        declaration.definition,
        header.signature,
        LangItem::Panic,
        None,
    )
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
    let ty = facts
        .definitions
        .get(definition.0 as usize)
        .copied()
        .flatten()
        .ok_or("imported panic role lost its installed declaration type")?;
    let Some(Kind::FunctionItem {
        item,
        signature: callable,
        ..
    }) = facts.types.kind(ty)
    else {
        return Err("imported panic role lost its canonical function item");
    };
    if item.0 != definition.0 {
        return Err("imported panic role changed its function item identity");
    }
    signature(facts, definition, callable, role, None)
}

fn signature(
    facts: &Declarations,
    definition: DefId,
    callable: TypeId,
    role: LangItem,
    format_arguments: Option<DefId>,
) -> Result<(), &'static str> {
    let Some(Kind::Function {
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
        LangItem::Panic => {
            if !parameters.is_empty()
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
            if !parameters.is_empty()
                || !matches!(facts.types.kind(*parameter),
                Some(Kind::Adt(item, arguments)) if item.0 == declaration.0
                    && facts.types.arguments(arguments).is_some_and(|mut args|
                        matches!(args.next(), Some(Argument::Lifetime(_))) && args.next().is_none()))
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
