//! Binder substitution over unresolved type graphs preserves their inference variables.
use super::{Constructor, Context, Error, RegionId, Store, TermId, Value};
use crate::types::substitute::Bound;
use std::collections::HashMap;

impl Context {
    /// Replace one surrounding region binder while retaining unsolved type and
    /// const variables. Nested callable binders remain local to their signatures.
    /// # Errors
    /// Rejects invalid terms/binders and exhausted inference work.
    pub fn substitute_regions(
        &mut self,
        store: &mut Store,
        root: TermId,
        regions: &[RegionId],
    ) -> Result<TermId, Error> {
        self.account_work(regions.len())?;
        self.map_regions(store, root, &mut Bound::new(regions.to_vec()))
    }
    pub(super) fn shift_regions(
        &mut self,
        store: &mut Store,
        root: TermId,
        amount: u16,
    ) -> Result<TermId, Error> {
        if amount == 0 {
            return Ok(root);
        }
        self.map_regions(store, root, &mut Bound::shift(amount))
    }
    /// Share one binder mapping across an inferred graph and its caller environment.
    /// Inference variables keep their identity; local callable frames remain local.
    /// # Errors
    /// Rejects malformed identities, invalid binder mappings and exhausted work.
    pub fn map_regions(
        &mut self,
        store: &mut Store,
        root: TermId,
        bound: &mut Bound,
    ) -> Result<TermId, Error> {
        let root = self.root(root)?;
        let mut work = vec![((root, 0_u16), false)];
        let mut results = HashMap::new();
        let mut children = Vec::new();
        while let Some(((id, depth), finish)) = work.pop() {
            self.step()?;
            let id = self.root(id)?;
            if results.contains_key(&(id, depth)) {
                continue;
            }
            let term = self.terms[id.0 as usize];
            let kind = match term.value {
                Value::Variable(_) => {
                    results.insert((id, depth), id);
                    continue;
                }
                Value::Known(ty) => {
                    let before = bound.steps;
                    let ty = bound
                        .apply_inside(store, ty, depth)
                        .map_err(|_| Error::InvalidIdentity)?;
                    self.account_work(bound.steps.saturating_sub(before))?;
                    let value = self.known(store, ty)?;
                    results.insert((id, depth), value);
                    continue;
                }
                Value::Construct(kind) => kind,
            };
            let child_depth = if matches!(kind, Constructor::Function { .. }) {
                depth.checked_add(1).ok_or(Error::Overflow)?
            } else {
                depth
            };
            if !finish {
                self.account_work((term.end - term.start) as usize)?;
                work.push(((id, depth), true));
                work.extend(
                    self.edges[term.start as usize..term.end as usize]
                        .iter()
                        .rev()
                        .map(|child| ((*child, child_depth), false)),
                );
                continue;
            }
            children.clear();
            for index in term.start..term.end {
                let child = self.root(self.edges[index as usize])?;
                children.push(
                    *results
                        .get(&(child, child_depth))
                        .ok_or(Error::InvalidIdentity)?,
                );
            }
            let value = self.map_constructor_regions(store, kind, &children, |region| {
                bound
                    .region(region, depth)
                    .map_err(|_| Error::InvalidIdentity)
            })?;
            results.insert((id, depth), value);
        }
        results
            .get(&(root, 0))
            .copied()
            .ok_or(Error::InvalidIdentity)
    }
    pub(super) fn map_constructor_regions(
        &mut self,
        store: &Store,
        kind: Constructor,
        children: &[TermId],
        mut replace: impl FnMut(RegionId) -> Result<RegionId, Error>,
    ) -> Result<TermId, Error> {
        let kind = match kind {
            Constructor::Adt(item, args) => Constructor::AdtArgs(
                item,
                self.materialize_arguments(store, args, &mut Vec::new())?,
            ),
            Constructor::FunctionItem(item, args) => Constructor::FunctionItemArgs(
                item,
                self.materialize_arguments(store, args, &mut Vec::new())?,
            ),
            other => other,
        };
        if let Constructor::Reference { mutable, region } = kind {
            return self.construct(
                Constructor::Reference {
                    mutable,
                    region: replace(region)?,
                },
                children,
            );
        }
        let kind = if let Some((arguments, _)) = kind.application() {
            kind.with_arguments(self.argument_regions(arguments, replace)?)?
        } else {
            kind
        };
        self.construct(kind, children)
    }
}
