//! Assumptions instantiate a caller premise; requirements introduce fresh universal scopes.
use super::{Constraints, Error, Kind, ProofScope, RegionId};
use crate::types::{
    TypeId,
    outlives::Queries,
    parameters::{
        clauses::{Clause, QuantifiedId},
        predicates::Bound,
    },
};
use std::collections::HashMap;

impl Constraints<'_> {
    /// Consume the outlives part of a complete caller clause. Other bound kinds
    /// remain the responsibility of their own formation and satisfaction checks.
    /// This never introduces a requirement's local scope as a new assumption.
    /// # Errors
    /// Rejects unknown identities, escaping captures and exhausted proof work.
    pub fn assume_outlives_clause(
        &mut self,
        queries: &mut Queries<'_>,
        clause: Clause,
    ) -> Result<(), String> {
        self.assume_outlives_clauses(queries, [clause])
    }
    /// Elaborate caller premises together, walking shared composite subjects once
    /// per independent target. No structural cache carries caller permissions.
    /// # Errors
    /// Rejects malformed clauses, escaping captures and exhausted proof work.
    pub fn assume_outlives_clauses(
        &mut self,
        queries: &mut Queries<'_>,
        clauses: impl IntoIterator<Item = Clause>,
    ) -> Result<(), String> {
        let mut targets = HashMap::new();
        let mut groups: Vec<(RegionId, Vec<TypeId>)> = Vec::new();
        for clause in clauses {
            self.steps = self.steps.saturating_add(1);
            if self.steps > self.limit {
                return Err(Error::Budget.to_string());
            }
            if let Some((ty, shorter)) = self.caller_components(queries, clause)? {
                let index = *targets.entry(shorter).or_insert_with(|| {
                    groups.push((shorter, Vec::new()));
                    groups.len() - 1
                });
                groups[index].1.push(ty);
            }
        }
        for (shorter, roots) in groups {
            let (regions, parameters, projections) = queries.clause_components(&roots)?;
            for longer in regions {
                self.assume(longer, shorter).map_err(|e| e.to_string())?;
            }
            for parameter in parameters {
                self.assume_parameter(parameter, shorter)
                    .map_err(|e| e.to_string())?;
            }
            for projection in projections {
                self.assume_projection(projection, shorter)?;
            }
        }
        Ok(())
    }
    fn caller_components(
        &mut self,
        queries: &mut Queries<'_>,
        clause: Clause,
    ) -> Result<Option<(TypeId, RegionId)>, String> {
        let Some(clause) = clause.caller_outlives(self.types)? else {
            return Ok(None);
        };
        let id = match clause {
            Clause::Predicate(predicate) => {
                if let Bound::Outlives(shorter) = predicate.bound {
                    self.assume_type_outlives(queries, predicate.ty, shorter)?;
                }
                return Ok(None);
            }
            Clause::ForAll(id) => id,
        };
        let value = self
            .types
            .quantified(id)
            .ok_or("invalid outlives assumption clause")?;
        let Bound::Outlives(shorter) = value.predicate.bound else {
            return Ok(None);
        };
        let subject = self
            .types
            .kind(value.predicate.ty)
            .ok_or("invalid outlives assumption subject")?;
        if matches!(subject, Kind::Projection(_)) {
            self.assume_quantified_projection(id)
                .map_err(|e| e.to_string())?;
            return Ok(None);
        }
        let shorter = match shorter.bound_parts() {
            Some((0, _)) => return Err("unelaborated local outlives assumption target".into()),
            Some((depth, slot)) => RegionId::bound(depth - 1, slot)?,
            None => shorter,
        };
        Ok(Some((value.predicate.ty, shorter)))
    }
    fn assume_quantified_projection(&mut self, id: QuantifiedId) -> Result<(), Error> {
        self.steps = self.steps.saturating_add(1);
        if self.steps > self.limit {
            return Err(Error::Budget);
        }
        let patterns = self
            .projected
            .get_or_insert_with(Default::default)
            .quantified
            .get_or_insert_with(Default::default);
        let before = patterns.steps();
        let result = patterns.assume(self.types, id);
        self.steps = self
            .steps
            .saturating_add(patterns.steps().saturating_sub(before));
        if self.steps > self.limit {
            return Err(Error::Budget);
        }
        result
    }
    /// Require a complete outlives clause without promoting any of it to premises.
    /// Empty binders still preserve the coordinate shift of enclosing captures.
    /// Other clause kinds are checked by their separate semantic consumers.
    /// # Errors
    /// Rejects unknown clauses, foreign identities and exhausted scoped proof work.
    pub fn require_outlives_clause(
        &mut self,
        queries: &mut Queries<'_>,
        clause: Clause,
        source: u32,
    ) -> Result<(), String> {
        let (predicate, count) = match clause {
            Clause::Predicate(predicate) => (predicate, None),
            Clause::ForAll(id) => {
                let value = self
                    .types
                    .quantified(id)
                    .ok_or("invalid required outlives clause")?;
                (value.predicate, Some(value.bound_regions))
            }
        };
        let Bound::Outlives(shorter) = predicate.bound else {
            return Ok(());
        };
        let scope = if let Some(count) = count {
            self.universal_scope(ProofScope::ROOT, count)
                .map_err(|e| e.to_string())?
        } else {
            ProofScope::ROOT
        };
        self.require_type(queries, predicate.ty, shorter, scope, source)
    }
}
