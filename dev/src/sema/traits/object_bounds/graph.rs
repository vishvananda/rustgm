//! Walk complete trait applications without storing transitive prefixes.
use crate::support::hash::HashSet;
use crate::{
    sema::model::Declarations,
    types::{
        Store,
        parameters::{
            clauses::{Clause, Comparison, Quantified, Quantifier},
            predicates::{Bound, Predicate},
        },
    },
};

pub(in crate::sema) struct Graph {
    work: Vec<Clause>,
    seen: HashSet<Comparison>,
    steps: usize,
    fixed: Option<u32>,
}
pub(in crate::sema) struct Visit {
    pub clause: Clause,
    pub predicate: Predicate,
    pub bound_regions: u32,
    pub implied: Vec<Clause>,
}
impl Graph {
    pub fn new(root: Clause) -> Self {
        Self {
            work: vec![root],
            seen: HashSet::default(),
            steps: 0,
            fixed: None,
        }
    }

    pub fn next(&mut self, facts: &mut Declarations) -> Result<Option<Visit>, &'static str> {
        while let Some(clause) = self.work.pop() {
            self.charge(1)?;
            let Clause::ForAll(id) = clause else {
                return Err("object graph requires a complete trait binder");
            };
            let quantified = quantified(&facts.types, clause)?;
            let fixed = if let Some(fixed) = self.fixed {
                fixed
            } else {
                let mut slots = crate::types::bound_uses::captures::Slots::default();
                let fixed = slots.predicate(&facts.types, quantified.predicate)?;
                self.charge(slots.steps)?;
                self.fixed = Some(fixed);
                fixed
            };
            let (key, work) = facts.types.clause_comparison_preserving(id, fixed)?;
            self.charge(work)?;
            if !self.seen.insert(key) {
                continue;
            }
            let requirements = facts
                .generics
                .traits
                .as_mut()
                .ok_or("missing object trait contracts")?
                .requirements(&mut facts.types, quantified.predicate)?;
            self.charge(requirements.implied.len())?;
            let mut compose = Quantifier::new(quantified.bound_regions)?;
            let mut implied = Vec::with_capacity(requirements.implied.len());
            for &clause in &requirements.implied {
                let clause = compose.clause(&mut facts.types, clause)?;
                if matches!(
                    quantified_value(&facts.types, clause)?.bound,
                    Bound::Trait(_) | Bound::ConstTrait(_) | Bound::MaybeConstTrait(_)
                ) {
                    self.work.push(clause);
                }
                implied.push(clause);
            }
            self.charge(compose.steps)?;
            return Ok(Some(Visit {
                clause,
                predicate: quantified.predicate,
                bound_regions: quantified.bound_regions,
                implied,
            }));
        }
        Ok(None)
    }

    pub(in crate::sema) fn fixed_slots(&self) -> Result<u32, &'static str> {
        self.fixed
            .ok_or("object graph has not entered its principal")
    }

    pub(super) fn steps(&self) -> usize {
        self.steps
    }

    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(count);
        if self.steps.saturating_add(self.work.len()) > 1_000_000 {
            return Err("object trait graph work budget exceeded");
        }
        Ok(())
    }
}
pub(in crate::sema) fn quantified_value(
    types: &Store,
    clause: Clause,
) -> Result<Predicate, &'static str> {
    quantified(types, clause).map(|q| q.predicate)
}
fn quantified(types: &Store, clause: Clause) -> Result<Quantified, &'static str> {
    let Clause::ForAll(id) = clause else {
        return Err("object graph requires a complete trait binder");
    };
    types.quantified(id).ok_or("invalid object graph binder")
}
