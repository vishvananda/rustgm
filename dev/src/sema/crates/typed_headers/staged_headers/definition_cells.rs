//! Only genuine typed declarations acquire cells; modules, traits and body locals do not.
use super::{Declarations, Error, Headers, Scope, error, mapped_source, s};
use crate::{
    resolve::model::DefId,
    types::{TypeId, arguments::Argument},
};

pub(super) fn prepare(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    facts: &mut Declarations,
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
    formal_cells(rows, graph, &mut cells, budget)?;
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
    for header in &rows.associated_types {
        assign(&mut cells, header.declaration, graph.ty(header.projection)?)?;
    }
    // An impl's declared-result definition cell is its value recipe; the
    // consumer re-derives every normalization and bound through it.
    for row in rows.associated_definitions() {
        assign(&mut cells, row.declaration, graph.ty(row.value)?)?;
    }
    for header in &rows.nominals {
        assign(&mut cells, header.declaration, graph.ty(header.self_type)?)?;
    }
    variant_cells(context, graph, facts, &mut cells, budget)?;
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
            s::DefinitionKind::Struct
                | s::DefinitionKind::Enum
                | s::DefinitionKind::Variant
                | s::DefinitionKind::AssociatedType
                | s::DefinitionKind::Function
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
/// A unit variant's cell is its enum's own type. A tuple variant's is the
/// constructor item over `fn(payload) -> Enum`, exactly as local declarations
/// complete it; the header supplies no signature.
fn variant_cells(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    facts: &mut Declarations,
    cells: &mut [Option<TypeId>],
    budget: &mut Headers,
) -> Result<(), Error> {
    let rows = context.descriptor.rows();
    for header in &rows.enums().headers {
        let Some(s::HeaderRef::Nominal(index)) = rows.header(header.declaration).map_err(error)?
        else {
            return Err(error("source enum cell lost its actual nominal header"));
        };
        let owner = graph.ty(rows.nominals[index as usize].self_type)?;
        let regions = nominal_regions(rows, rows.nominals[index as usize].self_type, budget)?;
        for variant in header
            .variants
            .slice(&rows.enums().variants)
            .map_err(error)?
        {
            budget.charge(1)?;
            let ty = match variant.shape {
                s::VariantShape::Unit => owner,
                s::VariantShape::Tuple => {
                    let payload = variant.fields.slice(&rows.enums().fields).map_err(error)?;
                    let signature = constructor_signature(
                        graph,
                        &mut facts.types,
                        (s::ConstructorPayload::Variant(payload), regions),
                        owner,
                        budget,
                    )?;
                    let item = context.definition(variant.declaration, budget)?;
                    facts
                        .types
                        .intern(crate::types::Kind::FunctionItem {
                            item: crate::types::ItemId(item.0),
                            signature,
                            arguments: crate::types::GenericArgs::EMPTY,
                        })
                        .map_err(error)?
                }
            };
            assign(cells, variant.declaration, ty)?;
        }
    }
    struct_cells(context, graph, facts, cells, budget)
}
/// Version thirteen: a unit struct constructor's cell is its struct's own
/// type, and a tuple struct's is the constructor item over `fn(fields) ->
/// Struct`, as local declarations complete them.
fn struct_cells(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    facts: &mut Declarations,
    cells: &mut [Option<TypeId>],
    budget: &mut Headers,
) -> Result<(), Error> {
    let rows = context.descriptor.rows();
    for (nominal, constructor) in struct_constructors(rows, budget)?.into_iter().enumerate() {
        let Some(constructor) = constructor else {
            continue;
        };
        let header = &rows.nominals[nominal];
        let owner = graph.ty(header.self_type)?;
        let fields = header.fields.slice(&rows.fields).map_err(error)?;
        let ty = if fields.is_empty() {
            owner
        } else {
            let regions = nominal_regions(rows, header.self_type, budget)?;
            let signature = constructor_signature(
                graph,
                &mut facts.types,
                (s::ConstructorPayload::Struct(fields), regions),
                owner,
                budget,
            )?;
            let item = context.definition(constructor, budget)?;
            facts
                .types
                .intern(crate::types::Kind::FunctionItem {
                    item: crate::types::ItemId(item.0),
                    signature,
                    arguments: crate::types::GenericArgs::EMPTY,
                })
                .map_err(error)?
        };
        assign(cells, constructor, ty)?;
    }
    Ok(())
}
/// Each validated struct constructor declaration, indexed by its struct's
/// nominal header row.
pub(super) fn struct_constructors(
    rows: s::ValidatedView<'_>,
    budget: &mut Headers,
) -> Result<Vec<Option<crate::support::crates::owner_catalog::DeclarationRef>>, Error> {
    budget.charge(rows.declarations.len().saturating_add(rows.nominals.len()))?;
    budget.reserve(rows.nominals.len().saturating_mul(size_of::<
        Option<crate::support::crates::owner_catalog::DeclarationRef>,
    >()))?;
    let mut constructors = vec![None; rows.nominals.len()];
    for (index, row) in rows.declarations.iter().enumerate() {
        if row.definition.kind != s::DefinitionKind::Variant {
            continue;
        }
        let declaration = crate::support::crates::owner_catalog::DeclarationRef(
            u32::try_from(index).map_err(|_| error("source constructor ordinal overflow"))?,
        );
        if let Some(s::HeaderRef::Constructor(nominal)) = rows.header(declaration).map_err(error)? {
            let cell = constructors
                .get_mut(nominal as usize)
                .ok_or_else(|| error("source struct constructor names an unknown struct"))?;
            if cell.replace(declaration).is_some() {
                return Err(error("source struct has a second constructor"));
            }
        }
    }
    Ok(constructors)
}
/// Each named type formal's cell is its canonical parameter type.
fn formal_cells(
    rows: s::ValidatedView<'_>,
    graph: &mapped_source::Graph,
    cells: &mut [Option<TypeId>],
    budget: &mut Headers,
) -> Result<(), Error> {
    for (index, slot) in rows.slots.iter().enumerate() {
        budget.charge(1)?;
        if let Some(binding) = slot.definition {
            let index =
                u32::try_from(index).map_err(|_| error("source formal slot reference overflow"))?;
            let ty = match graph.slot(s::SlotRef(index))? {
                Argument::Type(ty) => ty,
                // A version-thirteen nominal lifetime has no type cell.
                Argument::Lifetime(_)
                    if mapped_source::slot_bindings::nominal_lifetime(&rows.declarations, slot) =>
                {
                    continue;
                }
                _ => {
                    return Err(error(
                        "named source region cells require their wider declaration profile",
                    ));
                }
            };
            assign(cells, binding, ty)?;
        }
    }
    Ok(())
}
/// Version thirteen: a lifetime-generic nominal requires every body's storage
/// preparation, exactly as a local generic nominal does.
pub(super) fn regional_nominals(
    rows: s::ValidatedView<'_>,
    budget: &mut Headers,
) -> Result<bool, Error> {
    let mut regional = false;
    for header in &rows.nominals {
        regional |= nominal_regions(rows, header.self_type, budget)? != 0;
    }
    Ok(regional)
}
/// A version-thirteen nominal's lifetimes are the leading region arguments of
/// its own Self recipe, which validation fixed in slot order.
pub(super) fn nominal_regions(
    rows: s::ValidatedView<'_>,
    self_type: crate::support::crates::generics::TypeRef,
    budget: &mut Headers,
) -> Result<u32, Error> {
    let Some(s::Type::Nominal { arguments, .. }) = rows.types.get(self_type.0 as usize) else {
        return Err(error(
            "source enum cell lost its actual nominal Self recipe",
        ));
    };
    let arguments = arguments.slice(&rows.arguments).map_err(error)?;
    budget.charge(arguments.len())?;
    let regions = arguments
        .iter()
        .take_while(|argument| matches!(argument, s::Argument::Region(_)))
        .count();
    u32::try_from(regions).map_err(|_| error("source enum lifetime count overflow"))
}
/// A tuple variant's or tuple struct's constructor signature `fn(payload..)
/// -> Self`, read from its own payload rows; the cell and the callable
/// template share it. As a local constructor does, it binds its nominal's
/// lifetimes late.
pub(super) fn constructor_signature(
    graph: &mapped_source::Graph,
    types: &mut crate::types::Store,
    (payload, regions): (s::ConstructorPayload<'_>, u32),
    owner: TypeId,
    budget: &mut Headers,
) -> Result<TypeId, Error> {
    budget.charge(payload.len())?;
    let fields = (0..payload.len())
        .map(|index| {
            payload
                .ty(index)
                .ok_or_else(|| error("source constructor payload field is absent"))
                .and_then(|ty| graph.ty(ty))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let parameters = types.intern_list(&fields).map_err(error)?;
    types
        .intern(crate::types::Kind::Function {
            bound_regions: regions,
            parameters,
            result: owner,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
        })
        .map_err(error)
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
