//! Binder substitution over unresolved type graphs preserves their inference variables.
use super::{Constructor, Context, Error, RegionId, Store, TermId, Value};
use crate::types::substitute::Bound;
use std::collections::HashMap;

pub(super) struct MappedRegions {
    pub term: TermId,
    pub projections: bool,
}

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
    ) -> Result<MappedRegions, Error> {
        if amount == 0 {
            return Ok(MappedRegions {
                term: root,
                projections: false,
            });
        }
        self.map_regions_tracked(store, root, &mut Bound::shift(amount))
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
        self.map_regions_tracked(store, root, bound)
            .map(|mapped| mapped.term)
    }
    fn map_regions_tracked(
        &mut self,
        store: &mut Store,
        root: TermId,
        bound: &mut Bound,
    ) -> Result<MappedRegions, Error> {
        let mut mapping = Mapping::default();
        let term = mapping.apply(self, store, root, bound)?;
        Ok(MappedRegions {
            term,
            projections: mapping.projections,
        })
    }
    /// One immutable input graph may have many shared roots. Do not rebuild its
    /// prefixes separately for every exported input binding.
    pub(super) fn map_region_roots(
        &mut self,
        store: &mut Store,
        roots: &[TermId],
        bound: &mut Bound,
    ) -> Result<Vec<TermId>, Error> {
        self.account_work(roots.len())?;
        let mut mapping = Mapping::default();
        roots
            .iter()
            .map(|&root| mapping.apply(self, store, root, bound))
            .collect()
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

#[derive(Default)]
struct Mapping {
    results: HashMap<(TermId, u16), TermId>,
    work: Vec<((TermId, u16), bool)>,
    children: Vec<TermId>,
    projections: bool,
}
impl Mapping {
    fn apply(
        &mut self,
        context: &mut Context,
        store: &mut Store,
        root: TermId,
        bound: &mut Bound,
    ) -> Result<TermId, Error> {
        let root = context.root(root)?;
        self.work.push(((root, 0_u16), false));
        while let Some(((id, depth), finish)) = self.work.pop() {
            context.step()?;
            let id = context.root(id)?;
            if self.results.contains_key(&(id, depth)) {
                continue;
            }
            if bound.regions_only()
                && context
                    .completed(store, id)?
                    .is_some_and(|facts| !facts.regions)
            {
                self.results.insert((id, depth), id);
                continue;
            }
            let term = context.terms[id.0 as usize];
            let kind = match term.value {
                Value::Variable(_) => {
                    self.results.insert((id, depth), id);
                    continue;
                }
                Value::Known(ty) => {
                    self.projections |= store.contains_projections(ty) == Some(true);
                    let before = bound.steps;
                    let ty = bound
                        .apply_inside(store, ty, depth)
                        .map_err(|_| Error::InvalidIdentity)?;
                    context.account_work(bound.steps.saturating_sub(before))?;
                    let value = context.known(store, ty)?;
                    self.results.insert((id, depth), value);
                    continue;
                }
                Value::Construct(kind) => kind,
            };
            self.projections |= matches!(kind, Constructor::Projection(_));
            let child_depth = if matches!(kind, Constructor::Function { .. }) {
                depth.checked_add(1).ok_or(Error::Overflow)?
            } else {
                depth
            };
            if !finish {
                context.account_work((term.end - term.start) as usize)?;
                self.work.push(((id, depth), true));
                self.work.extend(
                    context.edges[term.start as usize..term.end as usize]
                        .iter()
                        .rev()
                        .map(|child| ((*child, child_depth), false)),
                );
                continue;
            }
            self.children.clear();
            for index in term.start..term.end {
                let child = context.root(context.edges[index as usize])?;
                self.children.push(
                    *self
                        .results
                        .get(&(child, child_depth))
                        .ok_or(Error::InvalidIdentity)?,
                );
            }
            let before = bound.steps;
            let value = context.map_constructor_regions(store, kind, &self.children, |region| {
                bound
                    .region(region, depth)
                    .map_err(|_| Error::InvalidIdentity)
            })?;
            context.account_work(bound.steps.saturating_sub(before))?;
            self.results.insert((id, depth), value);
        }
        let term = self
            .results
            .get(&(root, 0))
            .copied()
            .ok_or(Error::InvalidIdentity)?;
        Ok(term)
    }
}
