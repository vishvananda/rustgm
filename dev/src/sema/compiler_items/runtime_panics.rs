//! Defining-body context is authenticated separately from reached panic hooks.
use crate::{
    resolve::{expansion::Report, model::DefId, unit::NodeKey},
    sema::model::{Declarations, Function},
    support::bindings::LangItem,
    syntax::{
        ast::Kind as Syntax,
        item::{Abi, ItemKind},
    },
    types::{Kind, arguments::Argument, callable::Convention},
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
    signature(facts, *function, role)?;
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

fn signature(facts: &Declarations, function: Function, role: LangItem) -> Result<(), &'static str> {
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
        ..
    }) = facts.types.kind(function.signature)
    else {
        return Err("source panic role requires its safe Rust function signature");
    };
    let Some([parameter]) = facts.types.list(parameters) else {
        return Err("source panic role requires its actual single argument");
    };
    if facts.types.kind(result) != Some(Kind::Never)
        || !facts.is_const_function(function.definition)
    {
        return Err("source panic role requires a const declaration with never result");
    }
    let parameters = facts
        .generics
        .parameters
        .get(&function.definition)
        .map_or(&[][..], Vec::as_slice);
    match role {
        LangItem::PanicFmt => {
            let declaration = facts
                .source_bindings
                .as_deref()
                .and_then(|catalog| catalog.language(LangItem::FormatArguments))
                .ok_or("source panic_fmt requires actual format_arguments identity")?;
            if !parameters.is_empty()
                || !matches!(facts.types.kind(*parameter),
                Some(Kind::Adt(item, arguments)) if item.0 == declaration.definition.0
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
