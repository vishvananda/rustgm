//! Associated outputs determine parameters only after all projection inputs are known.
#[cfg(test)]
#[path = "../../../tests/invariants/impl_constraints.rs"]
mod tests;
use super::{
    ParameterId,
    predicates::{Bound, Predicate},
    uses::Uses,
};
use crate::types::{Kind, Store, TypeId, bound_uses};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Parameter {
    Value(ParameterId),
    Lifetime(u16),
}

struct Dependency {
    remaining: usize,
    outputs: HashSet<Parameter>,
}

pub struct Constraints {
    header: Predicate,
    known: HashSet<Parameter>,
    waiting: HashMap<Parameter, Vec<usize>>,
    dependencies: Vec<Dependency>,
    ready: Vec<usize>,
    steps: usize,
}

impl Constraints {
    /// Seed the impl header's injective positions, including its lifetime inputs.
    /// # Errors
    /// Rejects malformed types and exhausted traversal resources.
    pub fn new(store: &Store, header: Predicate, uses: Uses) -> Result<Self, &'static str> {
        let mut regions = bound_uses::Uses::constraining();
        regions.predicate(store, header)?;
        let steps = uses.steps.saturating_add(regions.steps);
        let known = parameters(uses, regions);
        let mut result = Self {
            header,
            known,
            waiting: HashMap::new(),
            dependencies: Vec::new(),
            ready: Vec::new(),
            steps,
        };
        result.account(0)?;
        Ok(result)
    }

    /// Add a directional dependency without proving the equality or inferring inputs.
    /// # Errors
    /// Rejects malformed projection identities and exhausted graph resources.
    pub fn equality(&mut self, store: &Store, predicate: Predicate) -> Result<(), &'static str> {
        let Bound::ProjectionEqual(result) = predicate.bound else {
            return Ok(());
        };
        let Some(Kind::Projection(application)) = store.kind(predicate.ty) else {
            return Err("impl constraint requires an unreduced associated equality");
        };
        if self.header
            == (Predicate {
                ty: application.subject,
                bound: Bound::Trait(application.trait_application),
            })
        {
            // The implementation being defined cannot determine its own outputs.
            return Ok(());
        }
        let mut outputs = self.occurrences(store, result, true)?;
        outputs.retain(|parameter| !self.known.contains(parameter));
        if outputs.is_empty() {
            return Ok(());
        }
        let mut inputs = self.occurrences(store, predicate.ty, false)?;
        inputs.retain(|parameter| !self.known.contains(parameter));
        self.account(inputs.len().saturating_add(outputs.len()).saturating_add(1))?;
        let index = self.dependencies.len();
        let remaining = inputs.len();
        self.dependencies.push(Dependency { remaining, outputs });
        for input in inputs {
            self.waiting.entry(input).or_default().push(index);
        }
        if remaining == 0 {
            self.ready.push(index);
        }
        Ok(())
    }

    /// Each newly determined parameter wakes only the equalities depending on it.
    /// Cycles without a header seed remain undetermined.
    /// # Errors
    /// Rejects exhaustion of the dependency traversal budget.
    pub fn solve(&mut self) -> Result<(), &'static str> {
        while let Some(index) = self.ready.pop() {
            let outputs = std::mem::take(&mut self.dependencies[index].outputs);
            self.account(outputs.len().saturating_add(1))?;
            for output in outputs {
                if !self.known.insert(output) {
                    continue;
                }
                for dependent in self.waiting.remove(&output).into_iter().flatten() {
                    self.account(1)?;
                    let dependency = &mut self.dependencies[dependent];
                    dependency.remaining -= 1;
                    if dependency.remaining == 0 {
                        self.ready.push(dependent);
                    }
                }
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn determines(&self, parameter: ParameterId) -> bool {
        self.known.contains(&Parameter::Value(parameter))
    }

    fn occurrences(
        &mut self,
        store: &Store,
        root: TypeId,
        constraining: bool,
    ) -> Result<HashSet<Parameter>, &'static str> {
        let (mut uses, mut regions) = if constraining {
            (Uses::constraining(), bound_uses::Uses::constraining())
        } else {
            (Uses::default(), bound_uses::Uses::default())
        };
        uses.collect(store, root)?;
        regions.collect(store, root)?;
        self.account(uses.steps.saturating_add(regions.steps))?;
        Ok(parameters(uses, regions))
    }

    fn account(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(count);
        if self.steps > 1_000_000 {
            return Err("impl parameter constraint work budget exceeded");
        }
        Ok(())
    }
}

fn parameters(uses: Uses, regions: bound_uses::Uses) -> HashSet<Parameter> {
    uses.parameters
        .into_iter()
        .chain(uses.constants)
        .map(Parameter::Value)
        .chain(regions.indices.into_iter().map(Parameter::Lifetime))
        .collect()
}
