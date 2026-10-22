//! Demand a fully determined inference subtree without applying numeric fallback.
use super::{Context, Domain, Error, Store, TermId, TypeId, Value};
use crate::support::hash::{HashMap, HashSet};
#[cfg(test)]
#[path = "../../../tests/invariants/settled_inputs.rs"]
mod settled_tests;

/// Scratch for one resolution pass with no intervening inference equations.
#[derive(Default)]
pub(super) struct Resolution {
    results: HashMap<TermId, TypeId>,
    active: HashSet<TermId>,
    work: Vec<(TermId, bool)>,
    children: Vec<TypeId>,
}
impl Resolution {
    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.map(&self.results);
        total.set(&self.active);
        total.vector(&self.work);
        total.vector(&self.children);
        total
    }

    fn clear(&mut self) {
        // Clearing a hash table visits its capacity. A wide query must not make
        // every later tiny request clear that same wide allocation. Both sets
        // contain nodes visited by this traversal, even when it failed early.
        let retained = self
            .results
            .len()
            .saturating_add(self.active.len())
            .saturating_mul(4)
            .max(64);
        if self.results.capacity() > retained {
            self.results = HashMap::default();
        } else {
            self.results.clear();
        }
        if self.active.capacity() > retained {
            self.active = HashSet::default();
        } else {
            self.active.clear();
        }
        self.work.clear();
        self.children.clear();
    }
}

impl Context {
    /// The unresolved variable domain restricts numeric trait candidate lookup.
    /// # Errors
    /// Rejects an invalid inference identity.
    pub fn unresolved_domain(&mut self, term: TermId) -> Result<Option<Domain>, Error> {
        let root = self.root(term)?;
        Ok(match self.terms[root.0 as usize].value {
            Value::Variable(domain) => Some(domain),
            Value::Known(_) | Value::Construct(_) => None,
        })
    }

    /// Apply numeric defaults only after ordinary equations and trait progress stop.
    /// General unknowns remain unknown; candidates must never default speculatively.
    /// # Errors
    /// Rejects exhausted inference work.
    pub fn default_numbers(&mut self) -> Result<bool, Error> {
        self.account_work(self.terms.len())?;
        let mut changed = false;
        let mut wake = Vec::new();
        for (index, term) in self.terms.iter_mut().enumerate() {
            if term.parent.0 as usize != index {
                continue;
            }
            let primitive = match term.value {
                Value::Variable(Domain::Integer) => super::Primitive::I32,
                Value::Variable(Domain::Float) => super::Primitive::F64,
                _ => continue,
            };
            term.value = Value::Known(TypeId::primitive(primitive));
            if self.dependencies.is_some() {
                wake.push(term.parent);
            }
            changed = true;
        }
        for id in wake {
            self.wake_resolution(super::dependencies::Dependency::Type(id))?;
        }
        Ok(changed)
    }
    /// Resolve only the requested reachable terms. A successful result cannot
    /// change under subsequent compatible type equations; regions remain facts.
    /// Completed subtrees without projections are immutable under compatible
    /// equations. Projection-bearing results remain local to one traversal.
    ///
    /// # Errors
    /// Rejects unresolved variables, cycles, invalid identities and work limits.
    pub fn resolved(&mut self, store: &mut Store, root: TermId) -> Result<TypeId, Error> {
        let root = self.root(root)?;
        if let Value::Known(ty) = self.terms[root.0 as usize].value {
            self.step()?;
            return Ok(ty);
        }
        // Transient results can contain projections or unfinished paths. Keep
        // those local; only the separately certified settled graph survives.
        let mut scratch = self.resolution.take().unwrap_or_default();
        let result = self.resolve_with(store, root, &mut scratch);
        scratch.clear();
        self.resolution = Some(scratch);
        result
    }

    pub(super) fn resolve_with(
        &mut self,
        store: &mut Store,
        root: TermId,
        resolution: &mut Resolution,
    ) -> Result<TypeId, Error> {
        let root = self.root(root)?;
        if let Value::Known(ty) = self.terms[root.0 as usize].value {
            self.step()?;
            return Ok(ty);
        }
        let Resolution {
            results,
            active,
            work,
            children,
        } = resolution;
        work.push((root, false));
        while let Some((id, finish)) = work.pop() {
            self.step()?;
            let id = self.root(id)?;
            if results.contains_key(&id) {
                continue;
            }
            if let Some(ty) = self.settled_type(id) {
                results.insert(id, ty);
                continue;
            }
            let term = self.terms[id.0 as usize];
            match term.value {
                Value::Variable(_) => return Err(Error::Unresolved),
                Value::Known(ty) => {
                    results.insert(id, ty);
                }
                Value::Construct(kind) if finish => {
                    children.clear();
                    for offset in term.start..term.end {
                        let child = self.root(self.edges[offset as usize])?;
                        children.push(*results.get(&child).ok_or(Error::InvalidIdentity)?);
                    }
                    let ty = self.canonical(store, kind, children)?;
                    self.remember_settled(store, id, ty)?;
                    results.insert(id, ty);
                    active.remove(&id);
                }
                Value::Construct(_) => {
                    if !active.insert(id) {
                        return Err(Error::Occurs);
                    }
                    work.push((id, true));
                    for child in self.edges[term.start as usize..term.end as usize]
                        .iter()
                        .rev()
                    {
                        work.push((*child, false));
                    }
                }
            }
        }
        results.get(&root).copied().ok_or(Error::InvalidIdentity)
    }

    pub(super) fn settled_type(&self, root: TermId) -> Option<TypeId> {
        match self.terms[root.0 as usize].value {
            Value::Construct(super::Constructor::Primitive(primitive)) => {
                Some(TypeId::primitive(primitive))
            }
            Value::Construct(super::Constructor::Never) => Some(TypeId::NEVER),
            _ => self.settled_facts(root)?.identity,
        }
    }

    pub(super) fn remember_settled(
        &mut self,
        store: &Store,
        root: TermId,
        ty: TypeId,
    ) -> Result<(), Error> {
        if matches!(
            self.terms[root.0 as usize].value,
            Value::Construct(super::Constructor::Primitive(_) | super::Constructor::Never)
        ) {
            return Ok(());
        }
        if let Some(completed) = super::completed::Completed::canonical(store, ty)? {
            self.set_settled(root, completed);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Primitive, infer::Constructor};

    #[test]
    fn small_queries_release_disproportionate_hash_storage() -> Result<(), Error> {
        let mut store = Store::default();
        let mut context = Context::default();
        let byte = context.known(&store, TypeId::primitive(Primitive::U8))?;
        let mut wide = Vec::new();
        for length in 0..1024 {
            wide.push(context.construct(Constructor::Array(length), &[byte])?);
        }
        let tuple = context.construct(Constructor::Tuple, &wide)?;
        context.resolved(&mut store, tuple)?;
        let scratch = context.resolution.as_ref().ok_or(Error::InvalidIdentity)?;
        assert!(scratch.results.capacity() >= 1024);
        let small = context.construct(Constructor::Array(1), &[byte])?;
        context.resolved(&mut store, small)?;
        let scratch = context.resolution.as_ref().ok_or(Error::InvalidIdentity)?;
        assert!(scratch.results.capacity() <= 64);
        assert!(scratch.active.capacity() <= 64);
        // Failed traversals must apply the same bound to their unfinished path.
        let hole = context.variable(Domain::Any)?;
        let mut deep = hole;
        for _ in 0..1024 {
            deep = context.construct(Constructor::Slice, &[deep])?;
        }
        assert_eq!(context.resolved(&mut store, deep), Err(Error::Unresolved));
        let scratch = context.resolution.as_ref().ok_or(Error::InvalidIdentity)?;
        assert!(scratch.active.capacity() >= 1024);
        assert_eq!(context.resolved(&mut store, hole), Err(Error::Unresolved));
        let scratch = context.resolution.as_ref().ok_or(Error::InvalidIdentity)?;
        assert!(scratch.active.capacity() <= 64);
        Ok(())
    }
}
