//! Source dereference declarations and demanded method contracts retain their owners.
use super::headers::{Flags, Trait};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    support::{bindings::LangItem, constness::Constness},
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
        .is_some_and(|catalog| catalog.role(LangItem::Deref).ok().flatten().is_some())
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
        .validated_role(LangItem::Deref)
        .map_err(error)?
        .ok_or_else(|| error("field discovery lacks its completed actual Deref"))?;
    let target = catalog
        .validated_role(LangItem::DerefTarget)
        .map_err(error)?
        .ok_or_else(|| error("field discovery lacks its completed actual Target"))?;
    let mut steps = 0;
    let header = super::headers::Trait::current(report, facts, base, &mut steps).map_err(error)?;
    crate::sema::header_attributes::charge_source_contract_query(facts, steps)?;
    header.canonical_inputs(facts).map_err(error)?;
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
        .validated_role(role)
        .map_err(error)?
        .ok_or_else(|| error("source dereference lacks its completed actual trait"))?;
    let base = catalog
        .validated_role(LangItem::Deref)
        .map_err(error)?
        .ok_or_else(|| error("source dereference lacks its completed actual Deref"))?;
    let target = catalog
        .validated_role(LangItem::DerefTarget)
        .map_err(error)?
        .ok_or_else(|| error("source dereference lacks its completed actual Target"))?;
    if target.parent != Some(base.definition) {
        return Err(error(
            "source dereference Target belongs to another actual trait",
        ));
    }
    let mut steps = 0;
    let header =
        super::headers::Trait::current(report, facts, declaration, &mut steps).map_err(error)?;
    header.canonical_inputs(facts).map_err(error)?;
    crate::sema::header_attributes::charge_source_contract_query(
        facts,
        8 + steps + header.members.len(),
    )?;
    let method = deref_method(report, &header, mutable).map_err(error)?;
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

fn deref_method(report: &Report, header: &Trait<'_>, mutable: bool) -> Result<DefId, &'static str> {
    let name = if mutable { "deref_mut" } else { "deref" };
    header
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
        .ok_or("source dereference lacks its actual named method")
}

pub(super) fn imported(
    report: &Report,
    facts: &Declarations,
    catalog: &super::Catalog,
    steps: &mut usize,
    complete: &mut super::roles::RoleSet,
) -> Result<(), &'static str> {
    let roles = [LangItem::Deref, LangItem::DerefTarget, LangItem::DerefMut];
    let [base, target, mutable] = roles.map(|role| catalog.role(role));
    let (base, target, mutable) = (base?, target?, mutable?);
    if [base, target, mutable].iter().all(Option::is_none) {
        return Ok(());
    }
    let (Some(base), Some(target)) = (base, target) else {
        return Err("imported dereference requires its actual Deref and Target declarations");
    };
    let header = checked_deref(report, facts, base, steps)?;
    let pointee = catalog
        .validated_role(LangItem::PointeeSized)?
        .ok_or("imported Deref requires actual PointeeSized")?;
    if header.members.len() != 2
        || header.written.supertraits != 1
        || !header.requires(&facts.types, pointee.definition, None, Constness::Ordinary)
        || target.parent != Some(base.definition)
        || facts.associated.trait_owner(target.definition) != Some(base.definition)
        || !header.members.contains(&target.definition)
        || facts
            .types
            .projection_schema(crate::types::ItemId(target.definition.0))
            .is_none_or(|schema| schema.trait_item.0 != base.definition.0)
    {
        return Err("imported Deref changed its actual Target/member/supertrait contract");
    }
    method_signature(
        facts,
        deref_method(report, &header, false)?,
        header.self_type,
        base.definition,
        target.definition,
        false,
    )?;
    let const_deref = header.flags.constant;
    complete.insert(LangItem::Deref);
    complete.insert(LangItem::DerefTarget);
    if let Some(mutable) = mutable {
        let header = checked_deref(report, facts, mutable, steps)?;
        let effect = if header.flags.constant {
            Constness::Conditional
        } else {
            Constness::Ordinary
        };
        if header.members.len() != 1
            || !header.requires(&facts.types, base.definition, None, effect)
            || (header.flags.constant
                && (!const_deref
                    || header.written.supertraits != 2
                    || !header.requires(
                        &facts.types,
                        pointee.definition,
                        None,
                        Constness::Ordinary,
                    )))
        {
            return Err("imported DerefMut changed its actual Deref supertrait or member contract");
        }
        method_signature(
            facts,
            deref_method(report, &header, true)?,
            header.self_type,
            base.definition,
            target.definition,
            true,
        )?;
        complete.insert(LangItem::DerefMut);
    }
    Ok(())
}

fn checked_deref<'a>(
    report: &'a Report,
    facts: &'a Declarations,
    role: super::Role,
    steps: &mut usize,
) -> Result<Trait<'a>, &'static str> {
    let header = Trait::current(report, facts, role, steps)?;
    header.profile(
        &facts.types,
        Flags {
            constant: header.flags.constant,
            unsafe_: false,
            auto_: false,
        },
        0,
    )?;
    header.canonical_inputs(facts)?;
    *steps = steps.saturating_add(16 + header.members.len() + header.inputs.len());
    Ok(header)
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
        .declaration_header(method)?
        .ok_or("source dereference method has no prepared header")?;
    let Some(Kind::Function {
        bound_regions,
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
    if header.declaration.source().is_none()
        && (header.body.is_some() || bound_regions != 1 || region.bound_parts() != Some((0, 0)))
    {
        return Err("imported dereference requires its actual abstract method and shared binder");
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
