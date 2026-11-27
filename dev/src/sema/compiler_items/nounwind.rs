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
        .role(LangItem::PanicNounwind)?
        .ok_or("source inhabited assertion requires the actual panic_nounwind declaration")?;
    let signature = if let Some(origin) = declaration.origin.source() {
        source_signature(report, facts, declaration, origin)?
    } else {
        let actual = facts
            .declaration_header(declaration.definition)?
            .ok_or("imported panic_nounwind requires its canonical declaration")?;
        if declaration.parent.is_some()
            || actual.declaration != declaration.origin
            || actual.body.is_none()
        {
            return Err("imported panic_nounwind changed its canonical declaration");
        }
        actual.signature
    };
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result: TypeId::NEVER,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
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
    let ty = facts.types.intern(Kind::FunctionItem {
        item: ItemId(declaration.definition.0),
        signature,
        arguments: GenericArgs::EMPTY,
    })?;
    Ok(Entry {
        instance: Instance::new(&facts.types, ty)?,
        signature,
        parameter,
    })
}

fn source_signature(
    report: &Report,
    facts: &Declarations,
    declaration: super::Role,
    origin: crate::resolve::unit::NodeKey,
) -> Result<TypeId, &'static str> {
    let function = facts
        .function_indices
        .get(&declaration.definition)
        .and_then(|&index| facts.functions.get(index))
        .copied()
        .ok_or("source panic_nounwind requires its completed function")?;
    let unit = report.package.unit(origin.file);
    let Syntax::Item(index) = unit.configured.ast.node(origin.node).kind else {
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
        || function.node != origin
        || function.body
            != Some(crate::resolve::unit::NodeKey {
                file: origin.file,
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
    let current = facts
        .header_attribute_contracts(report, declaration.definition)
        .map_err(|_| "source panic_nounwind lost its checked current header")?;
    if current
        .origin
        .source()
        .is_none_or(|s| s.declaration != origin)
    {
        return Err("source panic_nounwind header belongs to another declaration");
    }
    Ok(function.signature)
}
