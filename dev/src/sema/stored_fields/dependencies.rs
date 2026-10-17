//! Typed storage edges and raw projection leaves for one canonical type.
use super::{Concrete, Declarations, TypeId, graph::Registry};
use crate::types::{Kind, ListId, arguments::Argument};

#[derive(Default)]
pub(super) struct Dependencies {
    pub children: Vec<TypeId>,
    pub fields: Vec<(TypeId, TypeId)>,
    pub shape: Option<Vec<ListId>>,
}
pub(super) fn read(
    registry: &mut Registry,
    facts: &mut Declarations,
    ty: TypeId,
    session: &mut Concrete<'_>,
) -> Result<Dependencies, &'static str> {
    let mut edges = Dependencies::default();
    match facts
        .types
        .kind(ty)
        .ok_or("invalid storage dependency type")?
    {
        Kind::Adt(_, arguments) => {
            nominal(registry, facts, ty, session, &mut edges)?;
            let arguments = facts
                .types
                .arguments(arguments)
                .ok_or("invalid storage arguments")?;
            registry.charge(arguments.len())?;
            edges
                .children
                .extend(arguments.filter_map(|argument| match argument {
                    Argument::Type(ty) => Some(ty),
                    _ => None,
                }));
        }
        Kind::Tuple(list) => {
            let fields = facts.types.list(list).ok_or("invalid storage tuple")?;
            registry.charge(fields.len())?;
            edges.children.extend_from_slice(fields);
        }
        Kind::Array { element, .. } | Kind::ArrayTerm { element, .. } | Kind::Slice(element) => {
            edges.children.push(element);
        }
        // Referents and callable signatures do not own their nested storage.
        _ => {}
    }
    Ok(edges)
}
fn nominal(
    registry: &mut Registry,
    facts: &mut Declarations,
    ty: TypeId,
    session: &mut Concrete<'_>,
    edges: &mut Dependencies,
) -> Result<(), &'static str> {
    facts.aggregates.instantiate(&mut facts.types, ty)?;
    let projected = facts.types.contains_parameters(ty) == Some(false)
        && facts.aggregates.has_projected_fields(ty);
    let lists = facts
        .aggregates
        .raw_shape(&facts.types, ty)?
        .variants()
        .map(|variant| variant.fields)
        .collect::<Vec<_>>();
    let mut normalized = Vec::new();
    for list in lists {
        let fields = facts.types.list(list).ok_or("missing raw storage fields")?;
        registry.charge(fields.len().saturating_add(1))?;
        if !projected {
            edges.children.extend_from_slice(fields);
            continue;
        }
        let count = fields.len();
        let mut fields = Vec::with_capacity(count);
        for index in 0..count {
            let field = facts.types.list(list).ok_or("missing raw storage fields")?[index];
            let (normal, steps) = session.normalize(facts, field)?;
            registry.charge(steps)?;
            if facts.types.contains_projections(normal) != Some(false) {
                return Err("concrete storage retains an associated projection");
            }
            if facts.types.contains_projections(field) == Some(true) {
                edges.fields.push((field, normal));
            }
            fields.push(normal);
            edges.children.push(normal);
        }
        normalized.push(facts.types.intern_list(&fields)?);
    }
    if projected {
        edges.shape = Some(normalized);
    }
    Ok(())
}
