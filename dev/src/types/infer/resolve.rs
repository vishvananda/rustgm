//! Demand a fully determined inference subtree without applying numeric fallback.
use super::{Context, Error, Store, TermId, TypeId, Value};
use std::collections::{HashMap, HashSet};

impl Context {
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
