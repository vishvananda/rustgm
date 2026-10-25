//! Associated inputs use their own identities and one flattened captured lifetime scope.
use super::{Checker, DefId, DefKind, Error, NodeKey, OwnerKind};
use crate::sema::{associated::Family, regions};
#[cfg(test)]
#[path = "../../../../../tests/invariants/family_parameters.rs"]
mod tests;

impl Checker<'_> {
    pub(in crate::sema::declarations) fn associated_type_context(
        &mut self,
        id: DefId,
        key: NodeKey,
    ) -> Result<regions::Context, Error> {
        if let Some(&binder) = self.owner_binders.get(&id) {
            return Ok(regions::Context {
                binder: Some(binder),
                position: regions::Position::Output,
            });
        }
        let owner = self
            .facts
            .associated
            .owner(id)
            .and_then(|owner| self.facts.associated.get(owner))
            .ok_or_else(|| self.error(key, "missing associated type owner"))?;
        let source = self
            .owner_binders
            .get(&owner.self_definition)
            .copied()
            .ok_or_else(|| self.error(key, "missing associated type owner binder"))?;
        let item = self.item(key)?;
        if item.generics.start == item.generics.end
            && item.where_clause.start == item.where_clause.end
        {
            return Ok(regions::Context {
                binder: Some(source),
                position: regions::Position::Output,
            });
        }
        let declared = self.declared_regions(key, item.generics)?;
        let captures = self.regions.count(source);
        let binder = self.regions.capture(source, declared.clone());
        self.regions.associate(binder, id);
        for &parameter in &declared {
            self.regions
                .named(Some(binder), parameter)
                .map_err(|e| self.error(key, e))?;
        }
        let bounds = self.function_bounds(key, binder)?;
        self.facts.associated.families.insert(
            id,
            Family {
                captures,
                total: self.regions.count(binder),
                regions: bounds,
                requirements: None,
            },
        );
        self.facts.lifetimes.parameters.insert(id, declared);
        self.owner_binders.insert(id, binder);
        Ok(regions::Context {
            binder: Some(binder),
            position: regions::Position::Output,
        })
    }

    /// Trait family identities precede result bounds; impl families wait for their owner header.
    pub(in crate::sema::declarations) fn reserve_family_parameters(
        &mut self,
        traits: bool,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        for index in 0..self.facts.associated.members().len() {
            let member = self.facts.associated.members()[index];
            let definition = &self.report.resolution.definitions[member.definition.0 as usize];
            let Some(owner) = self
                .facts
                .associated
                .source_at(member.owner.0 as usize)
                .map_err(Error::unlocated)?
            else {
                continue;
            };
            if definition.kind != DefKind::TypeAlias
                || matches!(owner.kind, OwnerKind::Trait { .. }) != traits
                || self
                    .facts
                    .generics
                    .parameters
                    .contains_key(&member.definition)
            {
                continue;
            }
            let key = definition
                .node
                .ok_or_else(|| self.error(owner.source, "missing associated type source"))?;
            let context = self.associated_type_context(member.definition, key)?;
            if !self
                .facts
                .associated
                .families
                .contains_key(&member.definition)
            {
                continue;
            }
            self.declaration_parameters(member.definition, key, context)?;
            self.facts
                .generics
                .parameters
                .entry(member.definition)
                .or_default();
            self.drain(steps, limit)?;
        }
        Ok(())
    }
}
