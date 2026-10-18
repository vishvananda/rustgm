//! Declaration formation defers local-dependent inputs without granting caller permissions.
use super::{Contracts, Requirements};
use crate::types::{
    RegionId, Store, TypeId,
    arguments::Argument,
    bound_uses::captures::Captures,
    outlives::Pair,
    parameters::clauses::{Clause, QuantifiedId},
    substitute::Bound,
};
use std::{collections::HashMap, sync::Arc};

pub(in crate::sema) enum Formation {
    Ordinary(TypeId, Arc<Requirements>),
    Quantified(Arc<Inputs>),
}
pub(in crate::sema) struct Inputs {
    subject: Option<TypeId>,
    types: Vec<TypeId>,
    required: Vec<Clause>,
    regions: Vec<Pair>,
}
impl Formation {
    pub fn subject(&self) -> Option<TypeId> {
        match self {
            Self::Ordinary(ty, _) => Some(*ty),
            Self::Quantified(inputs) => inputs.subject,
        }
    }
    pub fn types(&self) -> &[TypeId] {
        match self {
            Self::Ordinary(_, requirements) => &requirements.types,
            Self::Quantified(inputs) => &inputs.types,
        }
    }
    pub fn required(&self) -> &[Clause] {
        match self {
            Self::Ordinary(_, requirements) => &requirements.required,
            Self::Quantified(inputs) => &inputs.required,
        }
    }
    pub fn regions(&self) -> &[Pair] {
        match self {
            Self::Ordinary(_, requirements) => &requirements.regions,
            Self::Quantified(inputs) => &inputs.regions,
        }
    }
}

pub(super) struct Cache {
    completed: HashMap<QuantifiedId, Arc<Inputs>>,
    captures: Captures,
    lower: Bound,
    work: usize,
}
impl Default for Cache {
    fn default() -> Self {
        Self {
            completed: HashMap::new(),
            captures: Captures::new(1),
            lower: Bound::lower(1),
            work: 0,
        }
    }
}
impl Contracts {
    /// Formation is distinct from using a trait bound or proving its implementation.
    /// A quantified input mentioning a local defers that whole input's formation;
    /// independent inputs and requirements are lifted to the surrounding caller.
    pub fn formation(
        &mut self,
        store: &mut Store,
        clause: Clause,
    ) -> Result<Formation, &'static str> {
        if self.steps > 1_000_000 {
            return Err("trait contract work budget exceeded");
        }
        let Clause::ForAll(id) = clause else {
            let predicate = clause.ordinary()?;
            return Ok(Formation::Ordinary(
                predicate.ty,
                self.requirements(store, predicate)?,
            ));
        };
        if let Some(found) = self
            .formation
            .as_ref()
            .and_then(|cache| cache.completed.get(&id))
        {
            return Ok(Formation::Quantified(Arc::clone(found)));
        }
        let value = store
            .quantified(id)
            .ok_or("invalid trait formation clause")?;
        let requirements = self.requirements(store, value.predicate)?;
        let cache = self.formation.get_or_insert_with(Default::default);
        let before = cache.steps();
        let result = cache.prepare(store, value.predicate.ty, &requirements);
        self.steps = self
            .steps
            .saturating_add(cache.steps().saturating_sub(before));
        if self.steps > 1_000_000 {
            return Err("trait contract work budget exceeded");
        }
        let result = Arc::new(result?);
        cache.completed.insert(id, Arc::clone(&result));
        Ok(Formation::Quantified(result))
    }
}
impl Cache {
    fn steps(&self) -> usize {
        self.work
            .saturating_add(self.captures.steps)
            .saturating_add(self.lower.steps)
    }
    fn input(&mut self, store: &mut Store, ty: TypeId) -> Result<Option<TypeId>, &'static str> {
        if self.captures.collect(store, ty)? {
            Ok(None)
        } else {
            self.lower.apply(store, ty).map(Some)
        }
    }
    fn prepare(
        &mut self,
        store: &mut Store,
        subject: TypeId,
        requirements: &Requirements,
    ) -> Result<Inputs, &'static str> {
        self.work = self.work.saturating_add(
            1 + requirements.types.len()
                + requirements.required.len()
                + 2 * requirements.regions.len(),
        );
        if self.steps() > 1_000_000 {
            return Err("trait formation work budget exceeded");
        }
        let mut result = Inputs {
            subject: self.input(store, subject)?,
            types: Vec::new(),
            required: Vec::new(),
            regions: Vec::new(),
        };
        for &ty in &requirements.types {
            if let Some(ty) = self.input(store, ty)? {
                result.types.push(ty);
            }
        }
        for &clause in &requirements.required {
            if !self.captures.clause(store, clause)? {
                result.required.push(self.lower.clause(store, clause)?);
            }
        }
        for &(longer, shorter) in &requirements.regions {
            if !local(longer) && !local(shorter) {
                let Argument::Lifetime(longer) = self
                    .lower
                    .apply_argument(store, Argument::Lifetime(longer))?
                else {
                    return Err("invalid formation lifetime");
                };
                let Argument::Lifetime(shorter) = self
                    .lower
                    .apply_argument(store, Argument::Lifetime(shorter))?
                else {
                    return Err("invalid formation lifetime");
                };
                result.regions.push((longer, shorter));
            }
        }
        Ok(result)
    }
}
fn local(region: RegionId) -> bool {
    region.bound_parts().is_some_and(|(depth, _)| depth == 0)
}
