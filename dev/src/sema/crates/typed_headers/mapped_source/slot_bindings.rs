//! Actual rigid owners are retained; absent lifetime definitions stay absent.
use super::{Argument, Context, Declarations, DefKind, Error, Headers, RegionId, error, s};
use crate::types::{
    ItemId, Kind,
    parameters::{Parameter, ParameterKind},
};

pub(in crate::sema::crates::typed_headers) fn kind(kind: s::DefinitionKind) -> DefKind {
    match kind {
        s::DefinitionKind::Module => DefKind::Module,
        s::DefinitionKind::Trait => DefKind::Trait,
        s::DefinitionKind::SelfType => DefKind::SelfType,
        s::DefinitionKind::Function => DefKind::Function,
        s::DefinitionKind::TypeParameter => DefKind::Generic,
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
                intern_type(&mut facts.types, owner, slot.index, sized.is_some())?
            }
            s::ParameterKind::Region => {
                let index = u16::try_from(slot.index).map_err(|_| {
                    error("source region slot exceeds canonical lifetime identity space")
                })?;
                Argument::Lifetime(RegionId::bound(0, index).map_err(error)?)
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
                let frame = rows
                    .binders
                    .get(binder.0 as usize)
                    .ok_or_else(|| error("source region lost its exact binder"))?;
                context.definition(frame.owner, budget)?;
                if slot >= frame.count {
                    return Err(error("source region exceeds its actual binder count"));
                }
                bound(depth, slot, frame.count)?
            }
            s::Region::Parameter { .. } | s::Region::Variable { .. } => {
                return Err(error(
                    "source owned/free regions require their wider explicit remap profile",
                ));
            }
        });
    }
    Ok(result)
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
