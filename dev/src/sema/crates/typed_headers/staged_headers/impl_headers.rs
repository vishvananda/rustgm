//! Genuine impl headers are requests for fresh conformance, never validated evidence.
use super::{Declarations, Error, Headers, error, mapped_source, predicate_remap, s};
use crate::{
    resolve::body_origin::ImportedDeclarationId,
    support::crates::owner_catalog::DeclarationRef,
    types::{TypeId, outlives::Pair, parameters::predicates::Predicate},
};

pub(super) struct Implementation {
    pub(super) owner: ImportedDeclarationId,
    pub(super) self_type: TypeId,
    /// `None` is a version-thirteen inherent impl on a provider nominal.
    pub(super) predicate: Option<Predicate>,
    pub(super) captures: u32,
    pub(super) bounds: Vec<Pair>,
}
pub(super) fn prepare(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    remap: &mut predicate_remap::Context<'_>,
    facts: &mut Declarations,
    declaration: DeclarationRef,
    budget: &mut Headers,
) -> Result<Implementation, Error> {
    let rows = context.descriptor.rows();
    budget.charge(1)?;
    let Some(s::HeaderRef::Implementation(index)) = rows.header(declaration).map_err(error)? else {
        return Err(error("source impl lacks its actual completed header"));
    };
    let raw = &rows.implementations[index as usize];
    if raw.self_definition != declaration
        || raw.scope != declaration
        || raw.constant
        || (raw.members.start != raw.members.end
            && !context.descriptor.profile().feature_predicates())
    {
        return Err(error(
            "source impl requires its wider conformance/effect profile",
        ));
    }
    let definition = context.definition(declaration, budget)?;
    let owner = context.catalog.declaration(definition).map_err(error)?;
    context
        .catalog
        .validate_declaration(owner, definition, crate::resolve::model::DefKind::SelfType)
        .map_err(error)?;
    let self_type = graph.ty(raw.self_type)?;
    let predicate = match raw.trait_application {
        Some(application) => Some(remap.application(application, &mut facts.types, budget)?),
        None if context.descriptor.profile().body_nodes() => None,
        None => {
            return Err(error(
                "source inherent impl requires its actual wider member consumer",
            ));
        }
    };
    if predicate.is_some_and(|predicate| predicate.ty != self_type) {
        return Err(error(
            "source impl changed its actual Self/trait application pair",
        ));
    }
    let relations = raw
        .region_bounds
        .slice(&rows.region_bounds)
        .map_err(error)?;
    budget.reserve(relations.len().saturating_mul(size_of::<Pair>()))?;
    let mut bounds = Vec::with_capacity(relations.len());
    for relation in relations {
        budget.charge(1)?;
        bounds.push((
            graph.region(relation.longer)?,
            graph.region(relation.shorter)?,
        ));
    }
    budget.observed.vector(&bounds);
    Ok(Implementation {
        owner,
        self_type,
        predicate,
        captures: raw.region_captures,
        bounds,
    })
}
