//! Only genuine typed declarations acquire cells; modules, traits and body locals do not.
use super::{Declarations, Error, Headers, Scope, error, mapped_source, s};
use crate::{
    resolve::model::DefId,
    types::{TypeId, arguments::Argument},
};

pub(super) fn prepare(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    facts: &Declarations,
    scopes: &[Scope],
    callables: &[super::template_ports::Callable],
    budget: &mut Headers,
) -> Result<Vec<(DefId, TypeId)>, Error> {
    let rows = context.descriptor.rows();
    budget.reserve(
        rows.declarations
            .len()
            .saturating_mul(size_of::<Option<TypeId>>() + size_of::<(DefId, TypeId)>()),
    )?;
    let mut cells = vec![None; rows.declarations.len()];
    for (index, slot) in rows.slots.iter().enumerate() {
        budget.charge(1)?;
        if let Some(binding) = slot.definition {
            let index =
                u32::try_from(index).map_err(|_| error("source formal slot reference overflow"))?;
            let Argument::Type(ty) = graph.slot(s::SlotRef(index))? else {
                return Err(error(
                    "named source region cells require their wider declaration profile",
                ));
            };
            assign(&mut cells, binding, ty)?;
        }
    }
    for (raw, scope) in rows.scopes.iter().zip(scopes) {
        budget.charge(1)?;
        if matches!(scope.inputs, super::Inputs::Implementation(_)) {
            let Some(s::HeaderRef::Implementation(index)) =
                rows.header(raw.owner).map_err(error)?
            else {
                return Err(error(
                    "source Self cell changed its actual implementation header kind",
                ));
            };
            assign(
                &mut cells,
                raw.owner,
                graph.ty(rows.implementations[index as usize].self_type)?,
            )?;
        }
    }
    for callable in callables {
        budget.charge(1)?;
        let id = context
            .catalog
            .declaration(callable.definition)
            .map_err(error)?;
        let (owner, row, _) = context.catalog.declaration_coordinate(id).map_err(error)?;
        if owner != context.descriptor.identity() {
            return Err(error("source callable cell has a foreign actual owner"));
        }
        assign(&mut cells, row, callable.ty)?;
    }
    let mut result = Vec::with_capacity(rows.declarations.len());
    for (index, (raw, cell)) in rows.declarations.iter().zip(cells).enumerate() {
        budget.charge(1)?;
        let required = matches!(
            raw.definition.kind,
            s::DefinitionKind::Function
                | s::DefinitionKind::SelfType
                | s::DefinitionKind::TypeParameter
        );
        if required != cell.is_some() {
            return Err(error(
                "source declaration has a missing or invented canonical type cell",
            ));
        }
        if let Some(ty) = cell {
            let row =
                u32::try_from(index).map_err(|_| error("source type cell ordinal overflow"))?;
            let definition = context.definition(
                crate::support::crates::owner_catalog::DeclarationRef(row),
                budget,
            )?;
            if facts
                .definitions
                .get(definition.0 as usize)
                .is_none_or(Option::is_some)
            {
                return Err(error(
                    "source type cell is outside its actual destination or already installed",
                ));
            }
            result.push((definition, ty));
        }
    }
    budget.charge(1)?;
    Ok(result)
}
fn assign(
    cells: &mut [Option<TypeId>],
    row: crate::support::crates::owner_catalog::DeclarationRef,
    ty: TypeId,
) -> Result<(), Error> {
    let cell = cells
        .get_mut(row.0 as usize)
        .ok_or_else(|| error("source type cell is outside its declaration table"))?;
    if cell.is_some_and(|actual| actual != ty) {
        return Err(error(
            "source type cell has conflicting actual owner/type mappings",
        ));
    }
    *cell = Some(ty);
    Ok(())
}
