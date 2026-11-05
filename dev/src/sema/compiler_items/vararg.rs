//! Actual vararg role applications remain separate from storage and call authority.
mod binding;
mod calls;
use crate::{
    resolve::bindings::Declaration,
    sema::model::Declarations,
    support::bindings::LangItem,
    types::{ItemId, Kind, RegionId, TypeId, arguments::Argument},
};
pub(crate) use binding::{BindingRequest, require as require_binding};
pub(crate) use calls::Read;

pub(crate) fn charge(facts: &Declarations, steps: usize) -> Result<(), &'static str> {
    crate::sema::header_attributes::charge_source_contract_query(facts, steps)
        .map_err(|_| "variadic preflight exceeds its existing source budget")
}

pub(crate) fn declaration(facts: &Declarations) -> Result<Declaration, &'static str> {
    crate::sema::header_attributes::charge_source_contract_query(facts, 2)
        .map_err(|_| "vararg source declaration query exceeded its budget")?;
    facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.language(LangItem::VaList))
        .ok_or("variadic definition requires the actual source VaList declaration")
}
pub(crate) fn binding_type(
    facts: &mut Declarations,
    region: RegionId,
) -> Result<TypeId, &'static str> {
    let declaration = declaration(facts)?;
    let template = facts
        .definitions
        .get(declaration.definition.0 as usize)
        .copied()
        .flatten()
        .ok_or("source VaList type is not complete")?;
    let (owner, _) = application(facts, template)?;
    let arguments = facts.types.intern_regions(&[region])?;
    let ty = facts.types.intern(Kind::Adt(owner, arguments))?;
    facts.aggregates.prepare(&mut facts.types, ty)?;
    let mut layouts = std::mem::take(&mut facts.layouts);
    let result = target(facts, &mut layouts, ty);
    facts.layouts = layouts;
    result?;
    Ok(ty)
}
pub(crate) fn application(
    facts: &Declarations,
    ty: TypeId,
) -> Result<(ItemId, RegionId), &'static str> {
    let declaration = declaration(facts)?;
    let Some(Kind::Adt(owner, arguments)) = facts.types.kind(ty) else {
        return Err("variadic list must be its actual source nominal application");
    };
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or("source VaList arguments are missing")?;
    let (Some(Argument::Lifetime(region)), None) = (arguments.next(), arguments.next()) else {
        return Err("demanded VaList requires its actual sole lifetime argument");
    };
    if owner.0 != declaration.definition.0
        || !facts
            .generics
            .nominal_contracts
            .contains(&declaration.definition)
    {
        return Err("variadic list lost its completed actual nominal header");
    }
    Ok((owner, region))
}

/// The x86-64 cursor contract is derived from completed actual source fields.
pub(crate) fn target(
    facts: &Declarations,
    layouts: &mut crate::abi::layout::Layouts,
    ty: TypeId,
) -> Result<(u64, u32), &'static str> {
    crate::sema::header_attributes::charge_source_contract_query(facts, 32)
        .map_err(|_| "variadic target query exceeded its existing source budget")?;
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("variadic target source catalog is missing")?;
    catalog
        .validated_language(LangItem::VaList)?
        .ok_or("variadic source nominal is not complete")?;
    let void = catalog
        .validated_language(LangItem::CVoid)?
        .ok_or("variadic target needs its actual c_void enum")?;
    let (owner, _) = application(facts, ty)?;
    let template = facts
        .definitions
        .get(owner.0 as usize)
        .copied()
        .flatten()
        .ok_or("variadic target lost its actual declaration template")?;
    let (_, region) = application(facts, template)?;
    let definition = facts
        .aggregates
        .get(owner)
        .ok_or("variadic target nominal fields are missing")?;
    if definition.kind != crate::types::aggregate::Kind::Struct
        || definition.repr.representation != crate::types::aggregate::Representation::Transparent
    {
        return Err("demanded VaList requires its actual transparent source struct");
    }
    let &[inner, marker] = fields(facts, template)? else {
        return Err("demanded VaList requires its actual cursor and lifetime-marker fields");
    };
    inner_shape(facts, inner, void.definition)?;
    let Some(Kind::Adt(_, arguments)) = facts.types.kind(marker) else {
        return Err("variadic target lost its actual nominal lifetime marker");
    };
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or("variadic marker arguments are missing")?;
    if arguments.next() != Some(Argument::Lifetime(region)) || arguments.next().is_some() {
        return Err("variadic target lost its actual nominal lifetime-marker application");
    }
    marker_chain(facts, marker, region)?;
    for field in [ty, inner, marker] {
        layouts
            .demand(&facts.types, &facts.aggregates, field)
            .map_err(|_| "variadic target requires its actual completed source layouts")?;
    }
    let outer = layouts
        .get(ty)
        .ok_or("variadic target layout disappeared")?;
    let raw = layouts
        .get(inner)
        .ok_or("variadic cursor layout disappeared")?;
    let marker = layouts
        .get(marker)
        .ok_or("variadic marker layout disappeared")?;
    if outer.size != 24
        || outer.align != 8
        || raw.size != 24
        || raw.align != 8
        || marker.size != 0
        || marker.align != 1
        || layouts
            .variants(outer)
            .first()
            .is_none_or(|variant| layouts.offsets(variant).first() != Some(&0))
        || layouts
            .variants(raw)
            .first()
            .is_none_or(|variant| layouts.offsets(variant) != [0, 4, 8, 16])
    {
        return Err("actual variadic source layouts differ from the supported x86-64 cursor ABI");
    }
    Ok((outer.size, outer.align))
}
fn fields(facts: &Declarations, ty: TypeId) -> Result<&[TypeId], &'static str> {
    let shape = facts.aggregates.shape(&facts.types, ty)?;
    let mut variants = shape.variants();
    let variant = variants
        .next()
        .ok_or("variadic nominal has no actual variant")?;
    if variants.next().is_some() {
        return Err("variadic source storage requires one actual struct variant");
    }
    facts
        .types
        .list(variant.fields)
        .ok_or("variadic source field types are missing")
}
fn marker_chain(facts: &Declarations, ty: TypeId, region: RegionId) -> Result<(), &'static str> {
    use crate::types::variance::Variance;
    let Some(Kind::Adt(owner, _)) = facts.types.kind(ty) else {
        return Err("variadic marker has no actual declaration");
    };
    if !facts
        .generics
        .nominal_contracts
        .contains(&crate::resolve::model::DefId(owner.0))
        || facts.lifetimes.variances.get(owner) != Some([Variance::Covariant].as_slice())
    {
        return Err("variadic marker requires its completed actual covariant lifetime declaration");
    }
    let &[covariant] = fields(facts, ty)? else {
        return Err("variadic marker requires its actual variance carrier field");
    };
    let Some(Kind::Adt(carrier, arguments)) = facts.types.kind(covariant) else {
        return Err("variadic marker lost its actual variance carrier owner");
    };
    if facts.lifetimes.variances.get(carrier) != Some([Variance::Covariant].as_slice())
        || !facts
            .generics
            .nominal_contracts
            .contains(&crate::resolve::model::DefId(carrier.0))
    {
        return Err("variadic carrier requires its completed actual covariant type declaration");
    }
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or("variadic carrier arguments are missing")?;
    let (Some(Argument::Type(reference)), None) = (arguments.next(), arguments.next()) else {
        return Err("variadic carrier requires its actual reference argument");
    };
    if facts.types.kind(reference)
        != Some(Kind::Reference {
            mutable: false,
            region,
            referent: TypeId::UNIT,
        })
    {
        return Err("variadic lifetime carrier changed its actual shared unit-reference argument");
    }
    let &[phantom] = fields(facts, covariant)? else {
        return Err("variadic carrier lost its actual phantom field");
    };
    let declaration = facts
        .source_bindings
        .as_deref()
        .and_then(|c| c.language(LangItem::PhantomData))
        .ok_or("variadic marker needs its real PhantomData declaration")?;
    let Some(Kind::Adt(owner, arguments)) = facts.types.kind(phantom) else {
        return Err("variadic phantom field has no nominal identity");
    };
    if owner.0 != declaration.definition.0 {
        return Err("variadic marker names another PhantomData declaration");
    }
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or("variadic phantom arguments are missing")?;
    let (Some(Argument::Type(callable)), None) = (arguments.next(), arguments.next()) else {
        return Err("variadic phantom field requires its actual function argument");
    };
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention,
    }) = facts.types.kind(callable)
    else {
        return Err("variadic marker requires its actual covariant function-return carrier");
    };
    if convention != crate::types::callable::Convention::RUST
        || facts.types.list(parameters) != Some(&[])
    {
        return Err("variadic marker function carrier changed its actual input or convention");
    }
    let result = super::intrinsic_contracts::item_roots(&facts.types, [result], |steps| {
        charge(facts, steps)
    })?
    .ok_or("variadic marker result relocation is incomplete")?;
    if result != [reference] {
        return Err("variadic marker return carrier changed its actual lifetime reference");
    }
    Ok(())
}
fn inner_shape(
    facts: &Declarations,
    ty: TypeId,
    void: crate::resolve::model::DefId,
) -> Result<(), &'static str> {
    let Some(Kind::Adt(owner, _)) = facts.types.kind(ty) else {
        return Err("variadic cursor requires its actual source nominal owner");
    };
    let definition = facts
        .aggregates
        .get(owner)
        .ok_or("variadic cursor fields are pending")?;
    if definition.kind != crate::types::aggregate::Kind::Struct
        || definition.repr.representation != crate::types::aggregate::Representation::C
        || definition.repr.arguments != crate::types::aggregate::ArgumentPassing::Indirect
    {
        return Err(
            "variadic cursor requires its source C layout and actual indirect argument policy",
        );
    }
    let &[gp, fp, overflow, saved] = fields(facts, ty)? else {
        return Err("variadic cursor requires its actual four source fields");
    };
    if gp != TypeId::primitive(crate::types::Primitive::I32) || fp != gp || overflow != saved {
        return Err("variadic cursor changed its source offset or pointer fields");
    }
    let Some(Kind::Pointer {
        mutable: false,
        pointee,
    }) = facts.types.kind(overflow)
    else {
        return Err("variadic cursor requires its actual const c_void pointers");
    };
    if !matches!(facts.types.kind(pointee),Some(Kind::Adt(owner,arguments)) if owner.0==void.0 && facts.types.arguments(arguments).is_some_and(|args|args.len()==0))
    {
        return Err("variadic cursor pointer names another c_void declaration");
    }
    Ok(())
}
