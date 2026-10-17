//! Associated constant types and values inherit their owner's immutable environment.
use super::{Checker, DefId, Error, ItemKind, NodeKey, regions};
use crate::{resolve::model::DefKind, sema::associated::OwnerKind, syntax::item::Visibility};

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
            return Ok(regions::Context::STATIC);
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
            position: regions::Position::Static,
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
            let key = definition.node.ok_or_else(|| {
                Error::at(
                    crate::resolve::unit::FileId(0),
                    0,
                    "missing constant declaration source",
                )
            })?;
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

    fn constant_type_regions(
        &self,
        id: DefId,
        key: NodeKey,
        ty: crate::types::TypeId,
    ) -> Result<(), Error> {
        let mut queries = crate::types::outlives::Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        let mut constraints = self
            .trait_region_environment(id, &mut queries)
            .map_err(|e| self.error(key, e))?;
        let requirements = queries
            .symbolic_requirements(ty)
            .map_err(|e| self.error(key, e))?;
        constraints
            .require_requirements(requirements, crate::types::relations::ProofScope::ROOT, 0)
            .map_err(|e| self.error(key, e))?;
        constraints.solve().map_err(|e| {
            self.error(
                key,
                format!("associated constant type has an unsatisfied lifetime requirement: {e}"),
            )
        })?;
        Ok(())
    }
}
