//! Build normalized input views while keeping original formation roots and shared holes.
use super::super::{Context, Error, Store, TermId, TypeId, Value};
use std::collections::HashMap;

#[derive(Default)]
struct Rewriting {
    results: HashMap<TermId, TermId>,
    work: Vec<(TermId, bool)>,
    children: Vec<TermId>,
}

impl Context {
    /// Share one transformed graph across every input of a predicate. Inference
    /// variables and original projection roots remain independent of this view.
    /// # Errors
    /// Rejects invalid replacements, equality predicates and exhausted work.
    pub fn rewritten_projection_inputs(
        &mut self,
        store: &Store,
        predicate: &super::super::predicates::Predicate,
        replacements: &HashMap<TermId, TypeId>,
    ) -> Result<super::super::predicates::Predicate, Error> {
        let mut rewriting = Rewriting::default();
        predicate.map_inputs(|root| {
            self.rewrite_projection_root(store, root, replacements, &mut rewriting)
        })
    }
    /// Replace independently proved projection roots without mutating the original
    /// input graph. Inference variables and const/lifetime identities remain shared.
    /// The caller owns each replacement's normalization and formation evidence.
    ///
    /// # Errors
    /// Rejects invalid identities, non-projection replacements and exhausted work.
    pub fn rewritten_projections(
        &mut self,
        store: &Store,
        root: TermId,
        replacements: &HashMap<TermId, TypeId>,
    ) -> Result<TermId, Error> {
        self.rewrite_projection_root(store, root, replacements, &mut Rewriting::default())
    }
    fn rewrite_projection_root(
        &mut self,
        store: &Store,
        root: TermId,
        replacements: &HashMap<TermId, TypeId>,
        rewriting: &mut Rewriting,
    ) -> Result<TermId, Error> {
        let root = self.root(root)?;
        let Rewriting {
            results,
            work,
            children,
        } = rewriting;
        work.push((root, false));
        while let Some((id, finish)) = work.pop() {
            self.step()?;
            let id = self.root(id)?;
            if results.contains_key(&id) {
                continue;
            }
            if let Some(&ty) = replacements.get(&id) {
                if !self.is_projection(store, id)? {
                    return Err(Error::InvalidIdentity);
                }
                results.insert(id, self.known(store, ty)?);
                continue;
            }
            match self.terms[id.0 as usize].value {
                Value::Variable(_) => {
                    results.insert(id, id);
                    continue;
                }
                Value::Known(ty) if store.contains_projections(ty) == Some(false) => {
                    results.insert(id, id);
                    continue;
                }
                _ => {}
            }
            let kind = self.materialize(store, id)?;
            let entry = self.terms[id.0 as usize];
            self.account_work((entry.end - entry.start) as usize)?;
            if !finish {
                work.push((id, true));
                work.extend(
                    self.edges[entry.start as usize..entry.end as usize]
                        .iter()
                        .rev()
                        .map(|&child| (child, false)),
                );
                continue;
            }
            children.clear();
            let mut changed = false;
            for offset in entry.start..entry.end {
                let child = self.root(self.edges[offset as usize])?;
                let rewritten = *results.get(&child).ok_or(Error::InvalidIdentity)?;
                changed |= child != rewritten;
                children.push(rewritten);
            }
            let result = if changed {
                self.construct(kind, children)?
            } else {
                id
            };
            results.insert(id, result);
        }
        results.get(&root).copied().ok_or(Error::InvalidIdentity)
    }
}
