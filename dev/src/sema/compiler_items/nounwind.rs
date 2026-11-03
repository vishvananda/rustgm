//! The real source panic entry retains its checked signature and ordinary body demand.
use crate::{
    resolve::expansion::Report,
    sema::model::Declarations,
    support::bindings::LangItem,
    syntax::{
        ast::Kind as Syntax,
        item::{Abi, ItemKind, Safety},
    },
    types::{
        GenericArgs, ItemId, Kind, Primitive, RegionId, TypeId, callable::Convention,
        instance::Instance,
    },
};
#[derive(Clone, Copy)]
pub(crate) struct Entry {
    pub instance: Instance,
    pub signature: TypeId,
    pub parameter: TypeId,
}
pub(super) fn prepare(report: &Report, facts: &mut Declarations) -> Result<Entry, &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source panic requires its defining catalog")?;
    catalog.require_provider_report(report)?;
    let declaration = catalog
        .language(LangItem::PanicNounwind)
        .ok_or("source inhabited assertion requires the actual panic_nounwind declaration")?;
    let function = facts
        .function_indices
        .get(&declaration.definition)
        .and_then(|&index| facts.functions.get(index))
        .copied()
        .ok_or("source panic_nounwind requires its completed function")?;
    let unit = report.package.unit(declaration.origin.file);
    let Syntax::Item(index) = unit.configured.ast.node(declaration.origin.node).kind else {
        return Err("source panic_nounwind lost its function item");
    };
    let item = unit.configured.ast.items[index as usize];
    let ItemKind::Function {
        flags,
        abi: Abi::Rust,
        body: Some(body),
        ..
    } = item.kind
    else {
        return Err("source panic_nounwind requires its actual defining Rust body");
    };
    if declaration.parent.is_some()
        || function.node != declaration.origin
        || function.body
            != Some(crate::resolve::unit::NodeKey {
                file: declaration.origin.file,
                node: body,
            })
        || flags.safety != Safety::Default
        || flags.asynchronous
        || !unit.configured.ast.list(item.generics).is_empty()
        || facts
            .generics
            .declared_predicates_in_scope(declaration.definition)
            .next()
            .is_some()
    {
        return Err("source panic_nounwind changed its defining nongeneric safe function");
    }
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result: TypeId::NEVER,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err("source panic_nounwind requires its safe Rust never-returning signature");
    };
    let Some(&[parameter]) = facts.types.list(parameters) else {
        return Err("source panic_nounwind requires exactly one message argument");
    };
    if !matches!(facts.types.kind(parameter), Some(Kind::Reference {
        mutable: false, referent, region: RegionId::STATIC,
    }) if referent == TypeId::primitive(Primitive::Str))
    {
        return Err("source panic_nounwind requires an actual static shared str parameter");
    }
    let current = facts
        .header_attribute_contracts(report, declaration.definition)
        .map_err(|_| "source panic_nounwind lost its checked current header")?;
    if current
        .origin
        .source()
        .is_none_or(|s| s.declaration != declaration.origin)
    {
        return Err("source panic_nounwind header belongs to another declaration");
    }
    let ty = facts.types.intern(Kind::FunctionItem {
        item: ItemId(declaration.definition.0),
        signature: function.signature,
        arguments: GenericArgs::EMPTY,
    })?;
    Ok(Entry {
        instance: Instance::new(&facts.types, ty)?,
        signature: function.signature,
        parameter,
    })
}
