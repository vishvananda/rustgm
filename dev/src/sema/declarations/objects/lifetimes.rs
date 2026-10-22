//! Object defaults distinguish lexical lifetimes from callable quantifiers.
use super::graph::{Graph, quantified_value};
use super::{Bound, Checker, Error, NodeKey, Pending, Predicate, RegionId, regions};
use crate::support::hash::HashSet;
use crate::types::{
    parameters::clauses::{Clause, Quantifier},
    substitute,
};

impl Checker<'_> {
    pub(super) fn prepare_object_guarantees(
        &mut self,
        key: NodeKey,
        principal: super::DefId,
    ) -> Result<(), Error> {
        let item = crate::types::ItemId(principal.0);
        if self.facts.types.dynamic_outlives_declared(item) {
            return Ok(());
        }
        let template = self
            .facts
            .generics
            .trait_self_bounds
            .get(&principal)
            .copied()
            .ok_or_else(|| self.error(key, "missing object lifetime principal template"))?;
        let Bound::Trait(application) = template.bound else {
            return Err(self.error(key, "object lifetime template requires a trait"));
        };
        let arguments = self
            .facts
            .types
            .trait_application(application)
            .ok_or_else(|| self.error(key, "invalid object lifetime template inputs"))?
            .arguments;
        let root = self.object_trait_clause(key, None, template)?;
        let mut graph = Graph::new(root);
        let mut regions = Vec::new();
        let mut seen = HashSet::default();
        let mut autos = 0_u8;
        while let Some(visit) = graph
            .next(&mut self.facts)
            .map_err(|e| self.error(key, e))?
        {
            for clause in visit.implied {
                let predicate =
                    quantified_value(&self.facts.types, clause).map_err(|e| self.error(key, e))?;
                if let Bound::Auto(marker) = predicate.bound
                    && predicate.ty == visit.predicate.ty
                {
                    autos |= 1 << marker as u8;
                }
                if let Bound::Outlives(region) = predicate.bound
                    && !matches!(region.bound_parts(), Some((0, _)))
                {
                    let region = substitute::Bound::lower(1)
                        .region(region, 0)
                        .map_err(|e| self.error(key, e))?;
                    if seen.insert(region) {
                        regions.push(region);
                    }
                }
            }
        }
        self.facts
            .types
            .declare_dynamic_outlives(item, arguments, &regions)
            .map_err(|e| self.error(key, e))?;
        self.facts
            .types
            .declare_dynamic_autos(item, autos)
            .map_err(|e| self.error(key, e))
    }
    pub(super) fn object_region(
        &mut self,
        key: NodeKey,
        pending: &Pending,
        predicate: Option<Predicate>,
        context: regions::Context,
    ) -> Result<RegionId, Error> {
        if let Some(region) = pending.region {
            return Ok(region);
        }
        if let Some(region) = self.container_object_region(key)? {
            return Ok(region);
        }
        let candidates = if let Some(predicate) = predicate {
            let scope = pending
                .principal
                .as_ref()
                .and_then(|principal| principal.scope);
            let clause = self.object_trait_clause(key, scope, predicate)?;
            self.object_lifetime_candidates(key, clause, context)?
        } else {
            HashSet::default()
        };
        if candidates.contains(&RegionId::STATIC) {
            return Ok(RegionId::STATIC);
        }
        if candidates.len() == 1
            && let Some(&region) = candidates.iter().next()
        {
            return Ok(region);
        }
        if candidates.len() > 1 {
            return Err(self.error(
                key,
                "ambiguous trait object lifetime requires an explicit bound",
            ));
        }
        if let Some(&region) = self.object_syntax.defaults.get(&key) {
            return Ok(region);
        }
        if matches!(
            context.position,
            regions::Position::Local(_) | regions::Position::Constant(_)
        ) {
            self.region(key, None, context)
        } else {
            Ok(RegionId::STATIC)
        }
    }

    pub(super) fn object_trait_clause(
        &mut self,
        key: NodeKey,
        scope: Option<super::Scope>,
        predicate: Predicate,
    ) -> Result<Clause, Error> {
        let clause = self.bound_clause(key, scope, predicate)?;
        if scope.is_some() {
            return Ok(clause);
        }
        let clause = substitute::Bound::shift(1)
            .clause(&mut self.facts.types, clause)
            .map_err(|e| self.error(key, e))?;
        Quantifier::new(0)
            .and_then(|mut q| q.clause(&mut self.facts.types, clause))
            .map_err(|e| self.error(key, e))
    }

    fn object_lifetime_candidates(
        &mut self,
        key: NodeKey,
        root: Clause,
        context: regions::Context,
    ) -> Result<HashSet<RegionId>, Error> {
        let mut candidates = HashSet::default();
        let mut graph = Graph::new(root);
        while let Some(visit) = graph
            .next(&mut self.facts)
            .map_err(|e| self.error(key, e))?
        {
            for clause in visit.implied {
                let value =
                    quantified_value(&self.facts.types, clause).map_err(|e| self.error(key, e))?;
                match value.bound {
                    Bound::Outlives(region) if !matches!(region.bound_parts(), Some((0, _))) => {
                        let region = substitute::Bound::lower(1)
                            .region(region, 0)
                            .map_err(|e| self.error(key, e))?;
                        if self.object_early_region(key, region, context)? {
                            candidates.insert(region);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(candidates)
    }
}
