//! Actual rigid owners are retained; absent lifetime definitions stay absent.
use super::{Argument, Context, Declarations, DefKind, Error, Headers, RegionId, error, s};
use crate::types::{
    ItemId, Kind,
    parameters::{Parameter, ParameterKind},
};

pub(in crate::sema::crates::typed_headers) fn kind(kind: s::DefinitionKind) -> DefKind {
    match kind {
        s::DefinitionKind::Module => DefKind::Module,
        s::DefinitionKind::Struct => DefKind::Struct,
        s::DefinitionKind::Enum => DefKind::Enum,
        s::DefinitionKind::Variant => DefKind::Variant,
        s::DefinitionKind::AssociatedType | s::DefinitionKind::TypeAlias => DefKind::TypeAlias,
        s::DefinitionKind::Trait => DefKind::Trait,
        s::DefinitionKind::TraitAlias => DefKind::TraitAlias,
        s::DefinitionKind::ForeignType => DefKind::ForeignType,
        s::DefinitionKind::SelfType => DefKind::SelfType,
        s::DefinitionKind::Function => DefKind::Function,
        s::DefinitionKind::Constant => DefKind::Const,
        s::DefinitionKind::Static => DefKind::Static,
        s::DefinitionKind::TypeParameter | s::DefinitionKind::ConstParameter => DefKind::Generic,
        s::DefinitionKind::RegionParameter => DefKind::Lifetime,
        s::DefinitionKind::ValueParameter => DefKind::Parameter,
        s::DefinitionKind::Local => DefKind::Local,
    }
}
pub(super) fn prepare(
    context: &Context<'_>,
    facts: &mut Declarations,
    budget: &mut Headers,
) -> Result<Vec<Argument>, Error> {
    let rows = context.descriptor.rows();
    budget.reserve(facts.types.parameter_allocation_bound(rows.slots.len()))?;
    let mut result = Vec::with_capacity(rows.slots.len());
    for slot in &rows.slots {
        budget.charge(1)?;
        let owner = context.definition(slot.owner, budget)?;
        if let Some(binding) = slot.definition {
            context.definition(binding, budget)?;
        }
        result.push(match slot.kind {
            s::ParameterKind::Type { sized } => {
                // The bit retains a declared shape requirement only. The actual
                // source Sized identity/full caller debt stays in the slot and
                // header inputs; this must never become builtin call permission.
                if let Some(address) = sized {
                    if address.artifact != rows.policy.provider {
                        return Err(error("source Sized formal belongs to another provider"));
                    }
                    let actual = context.definition(address.declaration, budget)?;
                    let id = context.catalog.declaration(actual).map_err(error)?;
                    context
                        .catalog
                        .validate_declaration(id, actual, DefKind::Trait)
                        .map_err(error)?;
                }
                intern_type(
                    &mut facts.types,
                    owner,
                    type_index(&rows.declarations, slot),
                    sized.is_some(),
                )?
            }
            s::ParameterKind::Region => {
                let index = u16::try_from(slot.index).map_err(|_| {
                    error("source region slot exceeds canonical lifetime identity space")
                })?;
                Argument::Lifetime(RegionId::bound(0, index).map_err(error)?)
            }
            s::ParameterKind::Const { ty } => {
                let ty = intern_const_type(
                    &mut facts.types,
                    s::const_formal_type(&rows.types, ty).map_err(error)?,
                    |row| context.definition(row, budget),
                )?;
                intern_const(
                    &mut facts.types,
                    owner,
                    type_index(&rows.declarations, slot),
                    ty,
                )?
            }
        });
    }
    Ok(result)
}
pub(super) fn regions(context: &Context<'_>, budget: &mut Headers) -> Result<Vec<RegionId>, Error> {
    let rows = context.descriptor.rows();
    let mut result = Vec::with_capacity(rows.regions.len());
    for region in &rows.regions {
        budget.charge(1)?;
        result.push(match *region {
            s::Region::Static => RegionId::STATIC,
            s::Region::Bound {
                binder,
                depth,
                slot,
            } => {
                let frame = bound_target(&rows.binders, binder, depth, || budget.charge(1))?;
                context.definition(frame.owner, budget)?;
                bound(depth, slot, frame.count)?
            }
            s::Region::Parameter { slot, depth } => {
                if let Some(region) =
                    nominal_region((&rows.slots, &rows.declarations), slot, depth)?
                {
                    region
                } else {
                    return Err(error(
                        "source owned/free regions require their wider explicit remap profile",
                    ));
                }
            }
            s::Region::Variable { .. } => {
                return Err(error(
                    "source owned/free regions require their wider explicit remap profile",
                ));
            }
        });
    }
    Ok(result)
}
/// Version thirteen: a struct or enum's own region slot, named in its own
/// declaration frame, is that nominal's template lifetime `bound(0, index)`,
/// exactly as a local nominal's field types name it. Other declared lifetimes
/// keep their wider profile.
pub(in crate::sema::crates::typed_headers) fn nominal_region(
    (slots, declarations): (&[s::ParameterSlot], &[s::Declaration]),
    slot: s::SlotRef,
    depth: u16,
) -> Result<Option<RegionId>, Error> {
    let row = slots
        .get(slot.0 as usize)
        .ok_or_else(|| error("source declared lifetime lost its actual slot"))?;
    if depth != 0 || !nominal_lifetime(declarations, row) {
        return Ok(None);
    }
    template_region(row).map(Some)
}
/// The wire stores the actual written lifetime in the declaration's template.
pub(in crate::sema::crates::typed_headers) fn template_lifetime(
    declarations: &[s::Declaration],
    slot: &s::ParameterSlot,
) -> bool {
    slot.kind == s::ParameterKind::Region
        && declarations
            .get(slot.owner.0 as usize)
            .is_some_and(|row| row.definition.kind.leading_regions())
}
pub(in crate::sema::crates::typed_headers) fn template_region(
    slot: &s::ParameterSlot,
) -> Result<RegionId, Error> {
    let index = u16::try_from(slot.index)
        .map_err(|_| error("source declared lifetime exceeds canonical identity space"))?;
    RegionId::bound(0, index).map_err(error)
}
/// A declaration's canonical parameter index counts its leading lifetimes.
/// Validated wire slots count those lifetimes separately from type slots.
pub(in crate::sema::crates::typed_headers) fn type_index(
    declarations: &[s::Declaration],
    slot: &s::ParameterSlot,
) -> u32 {
    if declarations
        .get(slot.owner.0 as usize)
        .is_some_and(|row| row.definition.kind.leading_regions())
    {
        slot.ordinal
    } else {
        slot.index
    }
}
/// Whether a slot's owner is a struct or enum, whose version-thirteen slots
/// lead with its lifetimes.
pub(in crate::sema::crates::typed_headers) fn nominal_owner(
    declarations: &[s::Declaration],
    owner: crate::support::crates::owner_catalog::DeclarationRef,
) -> bool {
    declarations.get(owner.0 as usize).is_some_and(|row| {
        matches!(
            row.definition.kind,
            s::DefinitionKind::Struct | s::DefinitionKind::Enum
        )
    })
}
/// A version-thirteen nominal's own lifetime slot.
pub(in crate::sema::crates::typed_headers) fn nominal_lifetime(
    declarations: &[s::Declaration],
    slot: &s::ParameterSlot,
) -> bool {
    slot.kind == s::ParameterKind::Region && nominal_owner(declarations, slot.owner)
}
pub(in crate::sema::crates::typed_headers) fn intern_type(
    store: &mut crate::types::Store,
    owner: crate::resolve::model::DefId,
    index: u32,
    sized: bool,
) -> Result<Argument, Error> {
    let id = store
        .intern_parameter(Parameter {
            owner: ItemId(owner.0),
            index,
            kind: ParameterKind::Type { sized },
        })
        .map_err(error)?;
    store
        .intern(Kind::Parameter(id))
        .map(Argument::Type)
        .map_err(error)
}
fn const_nominal(declaration: crate::resolve::model::DefId) -> Kind {
    Kind::Adt(ItemId(declaration.0), crate::types::GenericArgs::EMPTY)
}

/// Type node capacity is prepaid with the owning structural recipe table.
pub(in crate::sema::crates::typed_headers) fn intern_const_type(
    store: &mut crate::types::Store,
    ty: s::ConstFormalType,
    remap: impl FnOnce(
        crate::support::crates::owner_catalog::DeclarationRef,
    ) -> Result<crate::resolve::model::DefId, Error>,
) -> Result<crate::types::TypeId, Error> {
    match ty {
        s::ConstFormalType::Scalar(scalar) => Ok(crate::sema::crates::scalar(scalar)),
        s::ConstFormalType::Nominal(owner) => {
            store.intern(const_nominal(remap(owner)?)).map_err(error)
        }
    }
}

pub(in crate::sema::crates::typed_headers) fn find_const_type(
    store: &crate::types::Store,
    ty: s::ConstFormalType,
    remap: impl FnOnce(
        crate::support::crates::owner_catalog::DeclarationRef,
    ) -> Result<crate::resolve::model::DefId, Error>,
) -> Result<crate::types::TypeId, Error> {
    match ty {
        s::ConstFormalType::Scalar(scalar) => Ok(crate::sema::crates::scalar(scalar)),
        s::ConstFormalType::Nominal(owner) => store
            .find_kind(const_nominal(remap(owner)?))
            .ok_or_else(|| error("const formal lost its exact canonical nominal")),
    }
}

/// A rigid const parameter of `ty`, named by
/// its canonical const term exactly as a local const parameter is.
pub(in crate::sema::crates::typed_headers) fn intern_const(
    store: &mut crate::types::Store,
    owner: crate::resolve::model::DefId,
    index: u32,
    ty: crate::types::TypeId,
) -> Result<Argument, Error> {
    let id = store
        .intern_parameter(Parameter {
            owner: ItemId(owner.0),
            index,
            kind: ParameterKind::Const { ty },
        })
        .map_err(error)?;
    store
        .intern_const(crate::types::arguments::ConstKind::Parameter(id))
        .map(Argument::Const)
        .map_err(error)
}
pub(in crate::sema::crates::typed_headers) fn bound_target(
    binders: &[s::Binder],
    mut binder: crate::support::crates::generics::BinderRef,
    depth: u16,
    mut charge: impl FnMut() -> Result<(), Error>,
) -> Result<&s::Binder, Error> {
    let mut frame = binders
        .get(binder.0 as usize)
        .ok_or_else(|| error("source region lost its exact binder"))?;
    for _ in 0..depth {
        charge()?;
        binder = frame
            .parent
            .ok_or_else(|| error("source region escaped its enclosing binder"))?;
        frame = binders
            .get(binder.0 as usize)
            .ok_or_else(|| error("source region lost its enclosing binder"))?;
    }
    Ok(frame)
}
pub(in crate::sema::crates::typed_headers) fn bound(
    depth: u16,
    slot: u32,
    count: u32,
) -> Result<RegionId, Error> {
    if slot >= count {
        return Err(error("source region exceeds its actual binder count"));
    }
    let slot = u16::try_from(slot)
        .map_err(|_| error("source bound region slot exceeds canonical lifetime identity space"))?;
    RegionId::bound(depth, slot).map_err(error)
}
