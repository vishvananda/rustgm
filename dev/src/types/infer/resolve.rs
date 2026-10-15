//! Demand a fully determined inference subtree without applying numeric fallback.
use super::{Context, Domain, Error, Store, TermId, TypeId, Value};
use std::collections::{HashMap, HashSet};

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
    ///
    /// # Errors
    /// Rejects unresolved variables, cycles, invalid identities and work limits.
    pub fn resolved(&mut self, store: &mut Store, root: TermId) -> Result<TypeId, Error> {
        let root = self.root(root)?;
        let mut results = HashMap::new();
        let mut active = HashSet::new();
        let mut work = vec![(root, false)];
        let mut children = Vec::new();
        while let Some((id, finish)) = work.pop() {
            self.step()?;
            let id = self.root(id)?;
            if results.contains_key(&id) {
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
                    results.insert(id, self.canonical(store, kind, &children)?);
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
}
