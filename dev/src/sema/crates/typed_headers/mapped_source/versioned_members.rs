//! A v4 method plan authenticates actual installed rows before fresh conformance.
use super::{Graph, s};
use crate::{
    resolve::{
        body_origin::Catalog,
        model::{DefId, DefKind},
    },
    sema::{
        associated::{Inventory, Owner},
        model::Declarations,
    },
    types::{Store, arena::Identity, parameters::predicates::Predicate},
};
use std::rc::Rc;
pub(in crate::sema) struct PrimitiveMethods {
    arena: Identity,
    catalog: Rc<Catalog>,
    owner: DefId,
    predicate: Predicate,
    links: Vec<(DefId, Option<DefId>)>,
}
impl PrimitiveMethods {
    pub(in crate::sema) fn require(
        &self,
        facts: &Declarations,
        owner: DefId,
        predicate: Predicate,
    ) -> Result<(), &'static str> {
        if self.owner != owner
            || self.predicate != predicate
            || !facts.types.has_identity(&self.arena)
            || facts
                .crates
                .as_deref()
                .and_then(|linkage| linkage.origin_catalog())
                .is_none_or(|catalog| !std::ptr::eq(catalog, self.catalog.as_ref()))
        {
            return Err("primitive method conformance plan changed its actual arena/catalog/impl");
        }
        Ok(())
    }
    pub(in crate::sema) fn links(&self) -> &[(DefId, Option<DefId>)] {
        &self.links
    }
    pub(in crate::sema) fn into_links(self) -> Vec<(DefId, Option<DefId>)> {
        self.links
    }
}
impl Graph {
    pub(in crate::sema::crates::typed_headers) fn primitive_methods(
        &self,
        store: &Store,
        associated: &Inventory,
        owner: Owner,
        predicate: Predicate,
        budget: &mut super::super::Headers,
    ) -> Result<Option<PrimitiveMethods>, super::super::Error> {
        use super::super::error;
        self.require_store(store).map_err(error)?;
        self.owner(owner).map_err(error)?;
        let descriptor = self.descriptor(&self.origins).map_err(error)?;
        if descriptor.profile() != s::Profile::Source2024ModuleFeaturePredicates {
            return Ok(None);
        }
        let id = self
            .origins
            .declaration(owner.self_definition)
            .map_err(error)?;
        let (_, row, _) = self.origins.declaration_coordinate(id).map_err(error)?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Implementation(index)) = rows.header(row).map_err(error)? else {
            return Err(error(
                "v4 method plan lost its actual implementation header",
            ));
        };
        let raw = &rows.implementations[index as usize];
        if raw.members.start == raw.members.end {
            return Ok(None);
        }
        self.implementation(store, owner, predicate)
            .map_err(error)?;
        let schema = associated
            .schema(owner.self_definition)
            .ok_or_else(|| error("v4 method plan lost its installed Self schema"))?;
        let members = associated
            .owner(owner.self_definition)
            .and_then(|id| associated.owner_members(id))
            .ok_or_else(|| error("v4 method plan lost its actual member inventory"))?;
        let mut steps = 1;
        let result = self.associated_row(store, owner, schema, members, &mut steps);
        budget.charge(steps)?;
        result.map_err(error)?;
        let links = raw.members.slice(&rows.member_links).map_err(error)?;
        budget.reserve(
            links
                .len()
                .saturating_mul(4 * size_of::<(DefId, Option<DefId>)>())
                .saturating_add(size_of::<PrimitiveMethods>()),
        )?;
        let mut methods = Vec::with_capacity(links.len());
        for link in links {
            budget.charge(3)?;
            let member = link
                .trait_member
                .filter(|member| member.artifact == s::ArtifactRef(0))
                .ok_or_else(|| error("v4 method requires its genuine trait member"))?;
            let actual = link.implementation.ok_or_else(|| {
                error("v4 default method requires its wider body/conformance profile")
            })?;
            let member = self
                .origins
                .definition_at(descriptor.identity(), member.declaration)
                .map_err(error)?;
            let actual = self
                .origins
                .definition_at(descriptor.identity(), actual)
                .map_err(error)?;
            for definition in [member, actual] {
                let id = self.origins.declaration(definition).map_err(error)?;
                self.origins
                    .validate_declaration(id, definition, DefKind::Function)
                    .map_err(error)?;
            }
            methods.push((member, Some(actual)));
        }
        Ok(Some(PrimitiveMethods {
            arena: self.store.clone(),
            catalog: Rc::clone(&self.origins),
            owner: owner.self_definition,
            predicate,
            links: methods,
        }))
    }
}
