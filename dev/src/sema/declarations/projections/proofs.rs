//! Projected annotations retain their declaration or body proof until the owning gate.
use super::{Checker, DefId, Error, NodeKey, regions};
use crate::{
    sema::traits::obligations::Obligations,
    types::{
        RegionId, outlives::Queries, parameters::environment::Environment, relations::Constraints,
        substitute::Bound,
    },
};
use std::{collections::HashMap, sync::Arc};

impl Checker<'_> {
    pub(super) fn projection_environment(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<(Option<Arc<Environment>>, u32), Error> {
        let mut binder = context.binder;
        let mut depth = 0_u16;
        let mut environment = None;
        while let Some(current) = binder {
            if let Some(owner) = self.regions.owner(current) {
                environment = self.facts.generics.environments.get(&owner).cloned();
                break;
            }
            binder = self.regions.parent(current);
            depth = depth
                .checked_add(1)
                .ok_or_else(|| self.error(key, "projection binder depth exhausted"))?;
        }
        if depth != 0
            && let Some(env) = environment
        {
            environment = Some(Arc::new(
                env.substitute_regions(&mut self.facts.types, &mut Bound::shift(depth))
                    .map_err(|e| self.error(key, e))?,
            ));
        }
        Ok((environment, self.projection_region_start(key, context)?))
    }

    pub(in super::super) fn projection_region_start(
        &self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<u32, Error> {
        let first = if matches!(
            context.position,
            regions::Position::Local(_) | regions::Position::Constant(_)
        ) {
            u32::try_from(self.facts.body_regions.len())
                .ok()
                .and_then(|n| n.checked_add(1))
                .filter(|n| *n < 0x8000_0000)
                .ok_or_else(|| self.error(key, "projection region identity space exhausted"))?
        } else {
            self.projections
                .scopes
                .get(&context.binder)
                .map_or(1, |&index| self.projections.proofs[index].2.regions.end)
        };
        Ok(first)
    }

    pub(super) fn retain_projection_proof(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        proof: Obligations,
    ) -> Result<(), Error> {
        if proof.is_empty() {
            return Ok(());
        }
        if let Some(template) = self.projection_alias_owner(context) {
            let scope = self.projection_alias_scope(key, context, template)?;
            return self
                .facts
                .alias_requirements
                .direct(template.owner, template.parameters, proof, scope)
                .map_err(|e| self.error(key, e));
        }
        if let regions::Position::Local(body) | regions::Position::Constant(body) = context.position
        {
            if context
                .binder
                .is_some_and(|binder| self.regions.parent(binder).is_some())
            {
                return Err(
                    self.error(key, "local projection proof requires its callable universe")
                );
            }
            if matches!(context.position, regions::Position::Constant(_)) {
                self.facts
                    .constant_annotation_bodies
                    .insert(NodeKey { node: body, ..key });
            }
            proof
                .retain_body(&mut self.facts, NodeKey { node: body, ..key }, key)
                .map_err(|e| self.error(key, e))?;
        } else if let Some(&index) = self.projections.scopes.get(&context.binder) {
            self.projections.proofs[index]
                .2
                .extend(proof)
                .map_err(|e| self.error(key, e))?;
        } else {
            self.projections
                .scopes
                .insert(context.binder, self.projections.proofs.len());
            self.projections.proofs.push((key, context, proof));
        }
        Ok(())
    }

    pub(in super::super) fn validate_projection_regions(&mut self) -> Result<(), Error> {
        self.projections.scopes.clear();
        for (source, context, mut proof) in std::mem::take(&mut self.projections.proofs) {
            let (owner, count, mut fold) = self
                .projection_scope(context, &proof)
                .map_err(|e| self.error(source, e))?;
            proof
                .substitute(&mut self.facts.types, &mut fold)
                .map_err(|e| self.error(source, e))?;
            let mut queries = Queries::new(
                &self.facts.types,
                &self.facts.lifetimes.nominal_requirements,
                &self.facts.lifetimes.callables,
            );
            let mut constraints = if let Some(owner) = owner {
                self.trait_region_environment_count(owner, &mut queries, count)
            } else {
                Constraints::new(
                    &self.facts.types,
                    &self.facts.lifetimes.variances,
                    count,
                    1_000_000,
                )
                .map_err(|e| e.to_string())
            }
            .map_err(|e| self.error(source, e))?;
            proof
                .require(&mut constraints, &mut queries)
                .map_err(|e| self.error(source, e))?;
            constraints.solve().map_err(|e| {
                self.error(
                    source,
                    format!("associated type lifetime requirement is not satisfied: {e}"),
                )
            })?;
        }
        Ok(())
    }

    fn projection_scope(
        &self,
        context: regions::Context,
        proof: &Obligations,
    ) -> Result<(Option<DefId>, u32, Bound), &'static str> {
        let mut binders = Vec::new();
        let mut binder = context.binder;
        while let Some(current) = binder {
            binders.push(current);
            binder = self.regions.parent(current);
        }
        let owner = binders
            .last()
            .and_then(|&binder| self.regions.owner(binder));
        let mut mapping: HashMap<_, _> = proof
            .regions
            .clone()
            .map(|id| (RegionId(id), RegionId(id)))
            .collect();
        let mut count = 0_u32;
        for (depth, &binder) in binders.iter().enumerate().rev() {
            let depth = u16::try_from(depth).map_err(|_| "projection binder depth exhausted")?;
            for slot in 0..self.regions.count(binder) {
                let source = RegionId::bound(
                    depth,
                    u16::try_from(slot).map_err(|_| "projection binder size exhausted")?,
                )?;
                let target = RegionId::bound(
                    0,
                    u16::try_from(count).map_err(|_| "projection scope size exhausted")?,
                )?;
                mapping.insert(source, target);
                count += 1;
            }
        }
        Ok((owner, count, Bound::restore_regions(mapping)))
    }
}
