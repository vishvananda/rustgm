//! Reconstruct a checked object's principal, associated equalities and lifetime scopes.
use super::{Budget, Demand, Error, Kind, Port, Store, TypeId, Worklist, error, g, memo, s};
use crate::types::{ItemId, dynamic};

pub(super) fn next(
    rows: s::ValidatedView<'_>,
    reference: s::DynamicTypeRef,
    cursor: &mut usize,
    budget: &mut impl Budget,
) -> Result<Option<g::TypeRef>, Error> {
    let arguments = row(rows, reference)?
        .arguments
        .slice(&rows.arguments)
        .map_err(error)?;
    while let Some(argument) = arguments.get(*cursor) {
        budget.charge(1)?;
        *cursor += 1;
        if let s::Argument::Type(ty) = argument {
            return Ok(Some(*ty));
        }
    }
    Ok(None)
}

impl<M: memo::Storage> Worklist<M> {
    pub(super) fn dynamic_type<'rows, B: Budget, P: Port<'rows, B>>(
        &self,
        port: &mut P,
        store: &mut Store,
        reference: s::DynamicTypeRef,
        budget: &mut B,
    ) -> Result<Demand<TypeId>, Error> {
        let rows = port.rows();
        let row = row(rows, reference)?;
        let frame = rows
            .binders
            .get(row.binder.0 as usize)
            .filter(|frame| frame.kind == s::BinderKind::DynamicType)
            .ok_or_else(|| error("imported trait object lost its checked inner binder"))?;
        port.definition(frame.owner, budget)?;
        let region = match port.region(row.region, budget)? {
            Demand::Ready(region) => region,
            Demand::Blocked(dependency) => return Ok(Demand::Blocked(dependency)),
        };
        let arguments = match self.arguments(port, store, row.arguments, budget)? {
            Demand::Ready(arguments) => arguments,
            Demand::Blocked(dependency) => return Ok(Demand::Blocked(dependency)),
        };
        let principal = row
            .principal
            .map(|address| definition(port, address, budget))
            .transpose()?;
        let bindings = binding_rows(port, rows, row.bindings, budget)?;
        let markers = row
            .markers
            .slice(&rows.cold().dynamics.markers)
            .map_err(error)?;
        budget.charge(markers.len().saturating_add(1))?;
        budget.reserve(
            markers
                .len()
                .saturating_mul(4 * size_of::<crate::types::auto_traits::AutoTrait>())
                .saturating_add(4 * size_of::<dynamic::Shape>()),
        )?;
        let mut auto_traits = Vec::with_capacity(markers.len());
        for marker in markers {
            definition(port, marker.declaration, budget)?;
            auto_traits.push(marker.marker);
        }
        let shape = store
            .intern_dynamic_shape(dynamic::Bounds {
                principal,
                principal_arguments: row.principal_arguments,
                bindings: &bindings,
                auto_traits: &auto_traits,
            })
            .map_err(error)?;
        if !store.valid_dynamic_arguments(shape, arguments) {
            return Err(error(
                "imported trait object changed its complete argument shape",
            ));
        }
        store
            .intern(Kind::Dynamic {
                shape,
                arguments,
                region,
                bound_regions: frame.count,
            })
            .map(Demand::Ready)
            .map_err(error)
    }
}

fn row(rows: s::ValidatedView<'_>, reference: s::DynamicTypeRef) -> Result<s::DynamicType, Error> {
    rows.cold()
        .dynamics
        .objects
        .get(reference.0 as usize)
        .copied()
        .ok_or_else(|| error("imported trait object has no actual dynamic recipe"))
}
fn definition<'rows, B: Budget>(
    port: &impl Port<'rows, B>,
    address: s::DeclarationAddress,
    budget: &mut B,
) -> Result<ItemId, Error> {
    if address.artifact.0 != 0 {
        return Err(error(
            "imported object requires its checked dependency declaration map",
        ));
    }
    port.definition(address.declaration, budget)
        .map(|definition| ItemId(definition.0))
}
fn binding_rows<'rows, B: Budget>(
    port: &impl Port<'rows, B>,
    rows: s::ValidatedView<'rows>,
    range: g::Range,
    budget: &mut B,
) -> Result<Vec<dynamic::Binding>, Error> {
    let bindings = range.slice(&rows.cold().dynamics.bindings).map_err(error)?;
    budget.charge(bindings.len())?;
    budget.reserve(
        bindings
            .len()
            .saturating_mul(4 * size_of::<dynamic::Binding>()),
    )?;
    let mut result = Vec::with_capacity(bindings.len());
    for binding in bindings {
        result.push(dynamic::Binding {
            item: definition(port, binding.member, budget)?,
            trait_item: definition(port, binding.owner, budget)?,
            arguments: binding.arguments,
        });
    }
    Ok(result)
}
