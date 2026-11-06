//! Source dereference declarations and demanded method contracts retain their owners.
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    support::bindings::LangItem,
    types::{Kind, TypeId, callable::Convention},
};

pub(in crate::sema) struct Selected {
    pub trait_definition: DefId,
    pub method: DefId,
    pub signature: TypeId,
}

pub(in crate::sema) fn available(facts: &Declarations) -> bool {
    facts
        .source_bindings
        .as_deref()
        .is_some_and(|catalog| catalog.language(LangItem::Deref).is_some())
}

/// Discovery authenticates the actual projection, without selecting an unused
/// shared method or acquiring any call, const or mutable-place permission.
pub(in crate::sema) fn target(
    report: &Report,
    facts: &Declarations,
    source: NodeKey,
) -> Result<(DefId, crate::types::ItemId), Error> {
    let error = |message| Declarations::error(report, source, message);
    crate::sema::header_attributes::charge_source_contract_query(facts, 5)?;
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| error("field discovery lacks its actual source catalog"))?;
    catalog.require_provider_report(report).map_err(error)?;
    let base = catalog
        .validated_language(LangItem::Deref)
        .map_err(error)?
        .ok_or_else(|| error("field discovery lacks its completed actual Deref"))?;
    let target = catalog
        .validated_language(LangItem::DerefTarget)
        .map_err(error)?
        .ok_or_else(|| error("field discovery lacks its completed actual Target"))?;
    super::headers::Trait::source(report, facts, base)
        .map_err(error)?
        .canonical_inputs(facts)
        .map_err(error)?;
    let item = crate::types::ItemId(target.definition.0);
    if target.parent != Some(base.definition)
        || facts
            .types
            .projection_schema(item)
            .is_none_or(|schema| schema.trait_item.0 != base.definition.0)
    {
        return Err(error(
            "field discovery changed its actual Target owner or schema",
        ));
    }
    Ok((base.definition, item))
}

pub(in crate::sema) fn query(
    report: &Report,
    facts: &Declarations,
    source: NodeKey,
    mutable: bool,
) -> Result<Selected, Error> {
    let error = |message| Declarations::error(report, source, message);
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| error("source dereference lacks its actual catalog"))?;
    catalog.require_provider_report(report).map_err(error)?;
    let role = if mutable {
        LangItem::DerefMut
    } else {
        LangItem::Deref
    };
    let declaration = catalog
        .validated_language(role)
        .map_err(error)?
        .ok_or_else(|| error("source dereference lacks its completed actual trait"))?;
    let base = catalog
        .validated_language(LangItem::Deref)
        .map_err(error)?
        .ok_or_else(|| error("source dereference lacks its completed actual Deref"))?;
    let target = catalog
        .validated_language(LangItem::DerefTarget)
        .map_err(error)?
        .ok_or_else(|| error("source dereference lacks its completed actual Target"))?;
    if target.parent != Some(base.definition) {
        return Err(error(
            "source dereference Target belongs to another actual trait",
        ));
    }
    let header = super::headers::Trait::source(report, facts, declaration).map_err(error)?;
    header.canonical_inputs(facts).map_err(error)?;
    crate::sema::header_attributes::charge_source_contract_query(facts, 8 + header.members.len())?;
    let name = if mutable { "deref_mut" } else { "deref" };
    let method = header
        .members
        .iter()
        .copied()
        .find(|id| {
            report
                .resolution
                .definitions
                .get(id.0 as usize)
                .is_some_and(|d| {
                    d.kind == DefKind::Function && report.package.names.get(d.name) == name
                })
        })
        .ok_or_else(|| error("source dereference lacks its actual named method"))?;
    let signature = method_signature(
        facts,
        method,
        header.self_type,
        base.definition,
        target.definition,
        mutable,
    )
    .map_err(error)?;
    Ok(Selected {
        trait_definition: declaration.definition,
        method,
        signature,
    })
}

fn method_signature(
    facts: &Declarations,
    method: DefId,
    subject: TypeId,
    base: DefId,
    target: DefId,
    mutable: bool,
) -> Result<TypeId, &'static str> {
    let header = facts
        .function_header(method)?
        .ok_or("source dereference method has no prepared header")?;
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
        ..
    }) = facts.types.kind(header.signature)
    else {
        return Err("source dereference requires its actual safe Rust method");
    };
    if facts
        .generics
        .parameters
        .get(&method)
        .is_some_and(|p| !p.is_empty())
    {
        return Err("source dereference method generic profile is not implemented");
    }
    let [input] = facts
        .types
        .list(parameters)
        .ok_or("source dereference has no inputs")?
    else {
        return Err("source dereference requires one actual receiver");
    };
    let Some(Kind::Reference {
        mutable: input_mutable,
        region,
        referent,
    }) = facts.types.kind(*input)
    else {
        return Err("source dereference requires an actual borrowed receiver");
    };
    if input_mutable != mutable || referent != subject {
        return Err("source dereference requires the exact borrowed Self receiver");
    }
    let Some(Kind::Reference {
        mutable: result_mutable,
        region: output_region,
        referent,
    }) = facts.types.kind(result)
    else {
        return Err("source dereference requires its actual reference result");
    };
    let Some(Kind::Projection(projection)) = facts.types.kind(referent) else {
        return Err("source dereference requires its actual Target projection");
    };
    if result_mutable != mutable
        || output_region != region
        || projection.item.0 != target.0
        || projection.subject != subject
        || facts
            .types
            .trait_application(projection.trait_application)
            .is_none_or(|a| a.item.0 != base.0 || a.arguments != crate::types::GenericArgs::EMPTY)
        || projection.arguments != crate::types::GenericArgs::EMPTY
    {
        return Err("source dereference changed its actual Target or lifetime contract");
    }
    Ok(header.signature)
}
