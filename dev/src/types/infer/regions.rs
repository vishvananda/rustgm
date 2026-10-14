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
        self.map_regions(store, root, Bound::new(regions.to_vec()))
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
        self.map_regions(store, root, Bound::shift(amount))
    }
    fn map_regions(
        &mut self,
        store: &mut Store,
        root: TermId,
        mut bound: Bound,
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
            let value = self.region_constructor(store, kind, &children, &bound, depth)?;
            results.insert((id, depth), value);
        }
        results
            .get(&(root, 0))
            .copied()
            .ok_or(Error::InvalidIdentity)
    }
    fn region_constructor(
        &mut self,
        store: &mut Store,
        kind: Constructor,
        children: &[TermId],
        bound: &Bound,
        depth: u16,
    ) -> Result<TermId, Error> {
        let replace = |region| {
            bound
                .region(region, depth)
                .map_err(|_| Error::InvalidIdentity)
        };
        if let Constructor::Adt(item, args) | Constructor::FunctionItem(item, args) = kind {
            let args = store.region_args(args).ok_or(Error::InvalidIdentity)?;
            self.account_work(args.len())?;
            let args = args
                .iter()
                .copied()
                .map(replace)
                .collect::<Result<Vec<_>, _>>()?;
            let args = store.intern_regions(&args).map_err(|_| Error::Overflow)?;
            let kind = if matches!(kind, Constructor::Adt(..)) {
                Constructor::Adt(item, args)
            } else {
                Constructor::FunctionItem(item, args)
            };
            return self.construct(kind, children);
        }
        if let Constructor::Reference { mutable, region } = kind {
            return self.construct(
                Constructor::Reference {
                    mutable,
                    region: replace(region)?,
                },
                children,
            );
        }
        let kind = match kind {
            Constructor::AdtArgs(item, args) => {
                Constructor::AdtArgs(item, self.argument_regions(args, replace)?)
            }
            Constructor::FunctionItemArgs(item, args) => {
                Constructor::FunctionItemArgs(item, self.argument_regions(args, replace)?)
            }
            other => other,
        };
        self.construct(kind, children)
    }
}
