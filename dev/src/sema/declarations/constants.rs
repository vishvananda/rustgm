//! Associated constant types and values inherit their owner's immutable environment.
use super::{Checker, DefId, Error, ItemKind, NodeKey, regions};
use crate::{
    resolve::model::DefKind,
    sema::{associated::OwnerKind, values::Instance},
    support::crates::generics::source::ConstantBody,
    syntax::item::Visibility,
    types::GenericArgs,
};

impl Checker<'_> {
    pub(super) fn constant_context(
        &mut self,
        id: DefId,
        key: NodeKey,
    ) -> Result<regions::Context, Error> {
        let Some(owner) = self
            .facts
            .associated
            .owner(id)
            .and_then(|id| self.facts.associated.get(id))
            .copied()
        else {
            return Ok(regions::Context::item_static(id));
        };
        let item = self.item(key)?;
        if item.generics.start != item.generics.end
            || item.where_clause.start != item.where_clause.end
        {
            return Err(self.error(
                key,
                "associated constants cannot declare generic parameters or where clauses",
            ));
        }
        if !matches!(
            owner.kind,
            OwnerKind::Impl {
                trait_type: None,
                ..
            }
        ) && item.visibility != Visibility::Private
        {
            return Err(self.error(key, "trait constants require inherited visibility"));
        }
        let source = self
            .owner_binders
            .get(&owner.self_definition)
            .copied()
            .ok_or_else(|| self.error(key, "missing associated constant owner binder"))?;
        let binder = self.regions.capture(source, Vec::new());
        self.regions.associate(binder, id);
        self.constant_binders.insert(key, binder);
        self.owner_binders.insert(id, binder);
        // Constants introduce no parameters or bounds of their own. The owner's
        // completed environment is immutable and can be shared without elaboration.
        let parent = self
            .facts
            .generics
            .parents
            .get(&id)
            .copied()
            .ok_or_else(|| self.error(key, "missing associated constant generic owner"))?;
        if let Some(environment) = self.facts.generics.environments.get(&parent).cloned() {
            self.facts.generics.environments.insert(id, environment);
        }
        Ok(regions::Context {
            binder: Some(binder),
            position: regions::Position::Static(id),
        })
    }

    /// Required declarations have no initializer, but their types still need proof.
    pub(super) fn validate_constant_declarations(&mut self) -> Result<(), Error> {
        for index in 0..self.facts.associated.members().len() {
            let id = self.facts.associated.members()[index].definition;
            let definition = &self.report.resolution.definitions[id.0 as usize];
            if definition.kind != DefKind::Const {
                continue;
            }
            let Some(key) = definition.node else {
                // Imported constants already have a checked owner and type,
                // and closed inherent constants also have a completed value.
                if self.imported_inherent_constant(id)? || self.imported_range_bound(id)? {
                    continue;
                }
                if let Some((owner, body)) = self.imported_associated_constant(id)? {
                    self.validate_imported_constant(id, owner, body)?;
                    continue;
                }
                return Err(Error::at(
                    crate::resolve::unit::FileId(0),
                    0,
                    "missing constant declaration source",
                ));
            };
            let ty = self.facts.definitions[id.0 as usize]
                .ok_or_else(|| self.error(key, "missing constant declaration type"))?;
            if matches!(self.item(key)?.kind, ItemKind::Const { value: None, .. }) {
                let environment = self.facts.generics.environments.get(&id).cloned();
                let mut proof = self.declaration_proof(id, environment);
                if !proof
                    .check(&mut self.facts, ty)
                    .map_err(|e| self.error(key, e))?
                    .well_formed
                {
                    return Err(self.error(
                        key,
                        "associated constant declaration type must be well-formed",
                    ));
                }
                self.retain_trait_proof(id, key, &mut proof);
            }
            self.constant_type_regions(id, key, ty)?;
        }
        Ok(())
    }

    pub(super) fn constant_has_default(&self, definition: DefId) -> Result<bool, Error> {
        if let Some(key) = self.report.resolution.definitions[definition.0 as usize].node {
            return Ok(matches!(
                self.item(key)?.kind,
                ItemKind::Const { value: Some(_), .. }
            ));
        }
        self.imported_associated_constant(definition)?
            .filter(|(owner, _)| self.facts.associated.trait_owner(definition) == Some(*owner))
            .map(|(_, body)| matches!(body, ConstantBody::Original(_)))
            .ok_or_else(|| Error::unlocated("missing trait constant source"))
    }

    fn imported_associated_constant(
        &self,
        definition: DefId,
    ) -> Result<Option<(DefId, ConstantBody)>, Error> {
        self.facts
            .crates
            .as_deref()
            .map(|linkage| linkage.source_associated_constant(&self.facts, definition))
            .transpose()
            .map(Option::flatten)
            .map_err(Error::unlocated)
    }

    fn validate_imported_constant(
        &mut self,
        id: DefId,
        owner: DefId,
        body: ConstantBody,
    ) -> Result<(), Error> {
        let ty = self.facts.definitions[id.0 as usize]
            .ok_or_else(|| Error::unlocated("source constant has no type"))?;
        let environment = self.facts.generics.environments.get(&owner).cloned();
        let mut proof = self.declaration_proof(id, environment);
        let shape = proof.check(&mut self.facts, ty).map_err(Error::unlocated)?;
        if !shape.well_formed || (matches!(body, ConstantBody::Original(_)) && !shape.sized) {
            return Err(Error::unlocated(
                "associated constant declaration type must be well-formed",
            ));
        }
        self.constant_type_constraints(id, ty, proof.take_trait_proof())
            .map_err(Error::unlocated)
    }

    fn imported_inherent_constant(&self, definition: DefId) -> Result<bool, Error> {
        let Some(&owner) = self
            .facts
            .associated
            .owner(definition)
            .and_then(|id| self.facts.associated.get(id))
            .filter(|owner| matches!(owner.kind, OwnerKind::ImportedInherent(_)))
        else {
            return Ok(false);
        };
        self.facts
            .associated
            .require_imported(owner)
            .map_err(Error::unlocated)?;
        let instance = Instance {
            definition,
            arguments: GenericArgs::EMPTY,
        };
        Ok(self
            .facts
            .constant_instances
            .get(&instance)
            .and_then(|value| value.ty)
            .is_some_and(|ty| self.facts.definitions[definition.0 as usize] == Some(ty)))
    }

    fn imported_range_bound(&self, definition: DefId) -> Result<bool, Error> {
        use crate::support::bindings::LangItem;
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(false);
        };
        let Some(((), role)) = catalog
            .matching_role(
                definition,
                [((), LangItem::RangeMin), ((), LangItem::RangeMax)].into_iter(),
            )
            .map_err(Error::unlocated)?
        else {
            return Ok(false);
        };
        Ok(catalog
            .validated_role(role)
            .map_err(Error::unlocated)?
            .is_some_and(|bound| bound.definition == definition && bound.origin.source().is_none()))
    }

    fn constant_type_regions(
        &self,
        id: DefId,
        key: NodeKey,
        ty: crate::types::TypeId,
    ) -> Result<(), Error> {
        self.constant_type_constraints(id, ty, None)
            .map_err(|e| self.error(key, e))
    }

    fn constant_type_constraints(
        &self,
        id: DefId,
        ty: crate::types::TypeId,
        proof: Option<crate::sema::traits::obligations::Obligations>,
    ) -> Result<(), String> {
        let mut queries = crate::types::outlives::Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        let mut constraints = self.trait_region_environment(id, &mut queries)?;
        if let Some(proof) = proof {
            proof.require(&mut constraints, &mut queries)?;
        }
        let requirements = queries.symbolic_requirements(ty)?;
        constraints.require_requirements(
            requirements,
            crate::types::relations::ProofScope::ROOT,
            0,
        )?;
        constraints.solve().map_err(|e| {
            format!("associated constant type has an unsatisfied lifetime requirement: {e}")
        })?;
        Ok(())
    }
}
