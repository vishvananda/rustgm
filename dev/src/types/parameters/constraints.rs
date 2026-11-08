//! Associated outputs determine parameters only after all projection inputs are known.
#[cfg(test)]
#[path = "../../../tests/invariants/impl_constraints.rs"]
mod tests;
use super::{
    ParameterId,
    clauses::Clause,
    predicates::{Bound, Predicate},
    uses::Uses,
};
use crate::support::hash::{HashMap, HashSet};
use crate::types::{Kind, Store, TypeId, bound_uses};

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
            waiting: HashMap::default(),
            dependencies: Vec::new(),
            ready: Vec::new(),
            steps,
        };
        result.account(0)?;
        Ok(result)
    }

    /// Add a directional dependency without proving the equality or inferring
    /// inputs. A clause quantifier binds only lifetimes. Its own bound lifetimes are
    /// neither inputs nor outputs, the impl's lifetimes are read one binder
    /// deeper, and type and const parameters keep their identities. Non-type
    /// clauses determine nothing.
    /// # Errors
    /// Rejects unknown clauses, malformed projections and exhausted graph resources.
    pub fn clause(&mut self, store: &Store, clause: Clause) -> Result<(), &'static str> {
        match clause {
            Clause::Predicate(predicate) => self.dependency(store, predicate, 0),
            Clause::ForAll(id) => {
                let value = store
                    .quantified(id)
                    .ok_or("invalid impl constraint clause")?;
                self.dependency(store, value.predicate, 1)
            }
            Clause::UnstableFeature(id) => {
                store
                    .feature_name(id)
                    .ok_or("invalid non-type impl constraint clause")?;
                self.account(1)
            }
        }
    }

    fn dependency(
        &mut self,
        store: &Store,
        predicate: Predicate,
        depth: u16,
    ) -> Result<(), &'static str> {
        let Bound::ProjectionEqual(result) = predicate.bound else {
            return Ok(());
        };
        let Some(Kind::Projection(application)) = store.kind(predicate.ty) else {
            return Err("impl constraint requires an unreduced associated equality");
        };
        if self.projects_header(store, application, depth)? {
            // The implementation being defined cannot determine its own outputs.
            return Ok(());
        }
        let mut outputs = self.occurrences(store, result, true, depth)?;
        outputs.retain(|parameter| !self.known.contains(parameter));
        if outputs.is_empty() {
            return Ok(());
        }
        let mut inputs = self.occurrences(store, predicate.ty, false, depth)?;
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

    /// Outside a clause binder the header is compared exactly. Beneath one, the
    /// impl's lifetimes are shifted, so a header containing lifetimes matches any
    /// lifetime-bearing subject of its own trait. That can only reject more.
    fn projects_header(
        &self,
        store: &Store,
        application: crate::types::projection::Application,
        depth: u16,
    ) -> Result<bool, &'static str> {
        let own = Predicate {
            ty: application.subject,
            bound: Bound::Trait(application.trait_application),
        };
        if depth == 0 || self.header == own {
            return Ok(self.header == own);
        }
        let Some((header, _)) = self.header.bound.trait_parts() else {
            return Ok(false);
        };
        let item = |id| {
            store
                .trait_application(id)
                .map(|application| application.item)
                .ok_or("invalid impl constraint trait application")
        };
        if item(header)? != item(application.trait_application)? {
            return Ok(false);
        }
        if store
            .contains_regions(self.header.ty)
            .ok_or("invalid impl constraint header")?
        {
            return store
                .contains_regions(application.subject)
                .ok_or("invalid impl constraint subject");
        }
        Ok(application.subject == self.header.ty)
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

    /// rustc 1.98 `min_specialization`'s `unconstrained_parent_impl_args` reads
    /// a parent's associated equalities once, in declaration order, with no
    /// header seed or fixpoint. A parameter in a constraining position of an
    /// equality's result is determined unless that equality's projection, or an
    /// earlier equality's projection or result, names it. An equality that
    /// projects the header itself determines nothing.
    /// # Errors
    /// Rejects unknown clauses, malformed types and exhausted traversal resources.
    pub fn projection_determined(
        store: &Store,
        header: Predicate,
        clauses: impl IntoIterator<Item = Clause>,
    ) -> Result<Determined, &'static str> {
        let mut pass = Self {
            header,
            known: HashSet::default(),
            waiting: HashMap::default(),
            dependencies: Vec::new(),
            ready: Vec::new(),
            steps: 0,
        };
        let mut named = HashSet::default();
        for clause in clauses {
            pass.account(1)?;
            let (predicate, depth) = match clause {
                Clause::Predicate(predicate) => (predicate, 0),
                Clause::ForAll(id) => (
                    store
                        .quantified(id)
                        .ok_or("invalid impl constraint clause")?
                        .predicate,
                    1,
                ),
                Clause::UnstableFeature(_) => continue,
            };
            let Bound::ProjectionEqual(result) = predicate.bound else {
                continue;
            };
            if let Some(Kind::Projection(application)) = store.kind(predicate.ty)
                && pass.projects_header(store, application, depth)?
            {
                continue;
            }
            named.extend(pass.occurrences(store, predicate.ty, false, depth)?);
            for parameter in pass.occurrences(store, result, true, depth)? {
                if !named.contains(&parameter) {
                    pass.known.insert(parameter);
                }
            }
            named.extend(pass.occurrences(store, result, false, depth)?);
        }
        Ok(Determined {
            parameters: pass.known,
            steps: pass.steps,
        })
    }

    fn occurrences(
        &mut self,
        store: &Store,
        root: TypeId,
        constraining: bool,
        depth: u16,
    ) -> Result<HashSet<Parameter>, &'static str> {
        let (mut uses, mut regions) = if constraining {
            (Uses::constraining(), bound_uses::Uses::constraining())
        } else {
            (Uses::default(), bound_uses::Uses::default())
        };
        uses.collect(store, root)?;
        regions.collect_beneath(store, root, depth)?;
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

/// The parent parameters `Constraints::projection_determined` found, and its work.
pub struct Determined {
    parameters: HashSet<Parameter>,
    pub steps: usize,
}
impl Determined {
    #[must_use]
    pub fn lifetime(&self, slot: u16) -> bool {
        self.parameters.contains(&Parameter::Lifetime(slot))
    }
    #[must_use]
    pub fn value(&self, parameter: ParameterId) -> bool {
        self.parameters.contains(&Parameter::Value(parameter))
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
