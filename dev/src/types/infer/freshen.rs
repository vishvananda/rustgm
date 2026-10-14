//! Fresh free-region slots at inferred type boundaries; late binders stay shared.
use super::{Constructor, Context, Error, RegionId, Store, TermId, Value};
use crate::types::{GenericArgs, ItemId, Kind, callable::Templates, substitute::Bound};
use std::collections::HashSet;

enum Work {
    Enter(TermId, u16),
    Finish(TermId, Constructor, usize, u16),
}
#[derive(Default)]
pub(super) struct Scratch {
    work: Vec<Work>,
    values: Vec<TermId>,
    arguments: Vec<RegionId>,
    closed: HashSet<(TermId, u16)>,
}
impl Context {
    /// Give each free lifetime position a fresh inference variable. Array element
    /// types retain one slot; distinct tuple fields are distinct positions even
    /// when their source canonical type is shared. Function items keep their
    /// declaration identity and freshen coupled early arguments through templates.
    ///
    /// # Errors
    /// Rejects invalid identities and exhausted region/type work or storage.
    pub fn fresh_regions(
        &mut self,
        store: &mut Store,
        templates: &Templates,
        root: TermId,
        first: u32,
    ) -> Result<(TermId, u32), Error> {
        let root = self.root(root)?;
        if self.closed(store, root, 0)? {
            return Ok((root, 0));
        }
        let mut scratch = std::mem::take(&mut self.generalization);
        scratch.work.clear();
        scratch.values.clear();
        scratch.closed.clear();
        let result = self.fresh_walk(store, templates, root, first, &mut scratch);
        self.generalization = scratch;
        result
    }
    fn closed(&self, store: &Store, id: TermId, depth: u16) -> Result<bool, Error> {
        Ok(match self.terms[id.0 as usize].value {
            Value::Known(ty) => {
                store.has_free_regions(ty, depth) == Some(false)
                    || matches!(
                        store.kind(ty),
                        Some(Kind::FunctionItem {
                            arguments: GenericArgs::EMPTY,
                            ..
                        })
                    )
            }
            Value::Variable(_)
            | Value::Construct(
                Constructor::Primitive(_) | Constructor::Parameter(_) | Constructor::Never,
            ) => true,
            Value::Construct(Constructor::Adt(_, args) | Constructor::FunctionItem(_, args)) => {
                store
                    .region_args(args)
                    .ok_or(Error::InvalidIdentity)?
                    .is_empty()
            }
            Value::Construct(_) => false,
        })
    }
    fn fresh_walk(
        &mut self,
        store: &mut Store,
        templates: &Templates,
        root: TermId,
        first: u32,
        scratch: &mut Scratch,
    ) -> Result<(TermId, u32), Error> {
        let mut next = first;
        scratch.work.push(Work::Enter(root, 0));
        while let Some(work) = scratch.work.pop() {
            self.step()?;
            match work {
                Work::Enter(id, depth) => {
                    let id = self.root(id)?;
                    if self.closed(store, id, depth)? || scratch.closed.contains(&(id, depth)) {
                        scratch.values.push(id);
                        continue;
                    }
                    let original = self.materialize(store, id)?;
                    if let Constructor::FunctionItem(item, args) = original {
                        scratch.arguments.clear();
                        for region in store.region_args(args).ok_or(Error::InvalidIdentity)? {
                            self.step()?;
                            scratch.arguments.push(fresh(*region, depth, &mut next)?);
                        }
                        let value = self.fresh_item(store, templates, item, &scratch.arguments)?;
                        scratch.values.push(value);
                        continue;
                    }
                    let term = self.terms[id.0 as usize];
                    let kind = match original {
                        Constructor::Reference { mutable, region } => Constructor::Reference {
                            mutable,
                            region: fresh(region, depth, &mut next)?,
                        },
                        Constructor::Adt(item, arguments) => {
                            scratch.arguments.clear();
                            for region in
                                store.region_args(arguments).ok_or(Error::InvalidIdentity)?
                            {
                                self.step()?;
                                scratch.arguments.push(fresh(*region, depth, &mut next)?);
                            }
                            Constructor::Adt(
                                item,
                                store
                                    .intern_regions(&scratch.arguments)
                                    .map_err(|_| Error::Overflow)?,
                            )
                        }
                        _ => original,
                    };
                    let mark = scratch.values.len();
                    scratch.work.push(Work::Finish(id, kind, mark, depth));
                    let depth = if matches!(kind, Constructor::Function { .. }) {
                        depth.checked_add(1).ok_or(Error::Overflow)?
                    } else {
                        depth
                    };
                    for child in self.edges[term.start as usize..term.end as usize]
                        .iter()
                        .rev()
                    {
                        if self.steps.saturating_add(scratch.work.len()) >= 1_000_000 {
                            return Err(Error::Budget);
                        }
                        scratch.work.push(Work::Enter(*child, depth));
                    }
                }
                Work::Finish(id, kind, mark, depth) => {
                    let term = self.terms[id.0 as usize];
                    let mut unchanged =
                        matches!(term.value, Value::Construct(original) if original == kind);
                    for (offset, child) in (term.start..term.end).zip(&scratch.values[mark..]) {
                        unchanged &= self.root(self.edges[offset as usize])? == *child;
                    }
                    let result = if unchanged {
                        scratch.closed.insert((id, depth));
                        id
                    } else {
                        self.construct(kind, &scratch.values[mark..])?
                    };
                    scratch.values.truncate(mark);
                    scratch.values.push(result);
                }
            }
        }
        let result = scratch.values.pop().ok_or(Error::InvalidIdentity)?;
        if !scratch.values.is_empty() {
            return Err(Error::InvalidIdentity);
        }
        Ok((result, next.checked_sub(first).ok_or(Error::Overflow)?))
    }
    fn fresh_item(
        &mut self,
        store: &mut Store,
        templates: &Templates,
        item: ItemId,
        arguments: &[RegionId],
    ) -> Result<TermId, Error> {
        let template = templates.get(item).ok_or(Error::InvalidIdentity)?;
        if arguments.len() != template.parameters as usize {
            return Err(Error::InvalidIdentity);
        }
        let mut substitution = Bound::new(arguments.to_vec());
        let signature = substitution
            .apply(store, template.signature)
            .map_err(|_| Error::Budget)?;
        self.steps = self.steps.saturating_add(substitution.steps);
        self.step()?;
        let arguments = store
            .intern_regions(arguments)
            .map_err(|_| Error::Overflow)?;
        let ty = store
            .intern(Kind::FunctionItem {
                item,
                arguments,
                signature,
            })
            .map_err(|_| Error::Overflow)?;
        self.known(store, ty)
    }
}

fn fresh(region: RegionId, depth: u16, next: &mut u32) -> Result<RegionId, Error> {
    if region
        .bound_parts()
        .is_some_and(|(binder, _)| binder < depth)
    {
        return Ok(region);
    }
    let id = RegionId(*next);
    if id.0 == 0 || id.0 >= 0x8000_0000 {
        return Err(Error::Overflow);
    }
    *next = next.checked_add(1).ok_or(Error::Overflow)?;
    Ok(id)
}
