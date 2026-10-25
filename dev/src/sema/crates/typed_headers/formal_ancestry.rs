//! Collect only the demanded callable's genuine outer-to-inner formal path.
use super::super::super::{
    error,
    recipe_cursor::{Maps, Port, Reserved},
};
use crate::{
    resolve::{
        crates::{ReservationWork, unpaid_provided::RecipeView},
        package::Error,
    },
    support::crates::{generics::source as s, owner_catalog::DeclarationRef},
    types::{ItemId, Store, parameters::ParameterId},
};

pub(super) struct Formals {
    pub owners: Vec<ItemId>,
    pub parameters: Vec<ParameterId>,
}
pub(super) fn collect(
    view: RecipeView<'_>,
    scopes: &[Option<usize>],
    maps: &mut Maps,
    store: &mut Store,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<Formals, Error> {
    let mut port = Reserved::new(view, maps)?;
    let rows = port.rows();
    let actual = path(rows, scopes, owner, work)?;
    let path = actual.indices;
    let count = actual.slots;
    work.reserve(
        path.len()
            .saturating_mul(size_of::<ItemId>())
            .saturating_add(count.saturating_mul(size_of::<ParameterId>())),
    )?;
    let mut result = Formals {
        owners: Vec::with_capacity(path.len()),
        parameters: Vec::with_capacity(count),
    };
    for index in path.into_iter().rev() {
        work.charge(1)?;
        let scope = &rows.scopes[index];
        let definition = port.definition(scope.owner, work)?;
        result.owners.push(ItemId(definition.0));
        visit(&mut port, store, scope, definition, work, |parameter| {
            result.parameters.push(parameter)
        })?;
    }
    Ok(result)
}
pub(super) struct Path {
    pub indices: Vec<usize>,
    pub slots: usize,
}
pub(super) fn path(
    rows: s::ValidatedView<'_>,
    scopes: &[Option<usize>],
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<Path, Error> {
    let mut path = Vec::new();
    let mut next = Some(owner);
    let mut count = 0_usize;
    while let Some(current) = next {
        work.charge(1)?;
        if path.len() >= rows.scopes.len() {
            return Err(error("private callable ancestry is cyclic or incomplete"));
        }
        let index = scopes
            .get(current.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| error("private callable lacks its actual formal scope"))?;
        let scope = rows
            .scopes
            .get(index)
            .ok_or_else(|| error("private callable formal scope is outside its catalog"))?;
        if scope.owner != current {
            return Err(error(
                "private callable formal scope changed its actual owner",
            ));
        }
        let slots = scope.declared.slice(&rows.slots).map_err(error)?;
        work.charge(slots.len())?;
        count = count.saturating_add(slots.len());
        grow(&path, work)?;
        path.push(index);
        next = scope.parent;
    }
    Ok(Path {
        indices: path,
        slots: count,
    })
}
pub(super) fn visit(
    port: &mut Reserved<'_, '_>,
    store: &mut Store,
    scope: &s::ParameterScope,
    definition: crate::resolve::model::DefId,
    work: &mut ReservationWork,
    mut receive: impl FnMut(ParameterId),
) -> Result<(), Error> {
    let rows = port.rows();
    for (offset, slot) in scope
        .declared
        .slice(&rows.slots)
        .map_err(error)?
        .iter()
        .enumerate()
    {
        work.charge(1)?;
        if slot.owner != scope.owner {
            return Err(error(
                "private callable formal belongs to another actual owner",
            ));
        }
        if slot.kind == s::ParameterKind::Region {
            if slot.definition.is_some() {
                return Err(error(
                    "private callable declared lifetime requires its later scheme phase",
                ));
            }
            continue;
        }
        let binding = slot
            .definition
            .ok_or_else(|| error("private callable type formal omitted its actual declaration"))?;
        port.definition(binding, work)?;
        let offset = u32::try_from(offset)
            .map_err(|_| error("private callable formal reference overflow"))?;
        let reference = scope
            .declared
            .start
            .checked_add(offset)
            .ok_or_else(|| error("private callable formal reference overflow"))?;
        let argument = port.slot(store, s::SlotRef(reference), work)?;
        let parameter = crate::sema::generics::parameter_id(store, argument).map_err(error)?;
        if store
            .parameter(parameter)
            .is_none_or(|p| p.owner != ItemId(definition.0) || p.index != slot.index)
        {
            return Err(error(
                "private callable formal changed its canonical owner or argument position",
            ));
        }
        receive(parameter);
    }
    Ok(())
}
fn grow<T>(values: &Vec<T>, work: &mut ReservationWork) -> Result<(), Error> {
    if values.len() == values.capacity() {
        work.reserve(
            values
                .len()
                .saturating_add(1)
                .saturating_mul(4 * size_of::<T>()),
        )?;
    }
    Ok(())
}
