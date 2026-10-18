//! Fresh free-region slots at inferred type boundaries; late binders stay shared.
use super::arguments::{Argument, ArgumentsId, Slot};
use super::{Constructor, Context, Error, RegionId, Store, TermId, Value};
use crate::types::{GenericArgs, ItemId, Kind, callable::Templates, substitute::Bound};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FreshRegions {
    pub term: TermId,
    pub regions: u32,
    /// Rebuilt signatures may introduce new projection occurrences, even when
    /// no lifetime changed. Register the returned graph in its enclosing scope.
    /// False proves no new work; true does not grant normalization authority.
    pub needs_normalization: bool,
}

enum Work {
    Enter(TermId, u16),
    Finish(TermId, Constructor, usize, u16),
    Item(GenericItem, usize),
}
#[derive(Clone, Copy)]
struct GenericItem {
    item: ItemId,
    arguments: ArgumentsId,
    depth: u16,
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
    ) -> Result<FreshRegions, Error> {
        self.fresh_regions_with(store, templates, root, first, &mut |_, _| Ok(false))
    }
    /// Preserve inputs already proved invariant before allocating fresh regions.
    pub(crate) fn fresh_regions_with(
        &mut self,
        store: &mut Store,
        templates: &Templates,
        root: TermId,
        first: u32,
        rigid: &mut impl FnMut(&Store, ItemId) -> Result<bool, Error>,
    ) -> Result<FreshRegions, Error> {
        let root = self.root(root)?;
        if self.closed(store, root, 0)? {
            return Ok(FreshRegions {
                term: root,
                regions: 0,
                needs_normalization: false,
            });
        }
        let mut scratch = std::mem::take(&mut self.generalization);
        scratch.work.clear();
        scratch.values.clear();
        scratch.closed.clear();
        let result = self.fresh_walk(store, templates, root, first, &mut scratch, rigid);
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
    #[expect(
        clippy::too_many_lines,
        reason = "one worklist dispatch preserves binder depth and couples fresh item arguments to their rebuilt signatures"
    )]
    fn fresh_walk(
        &mut self,
        store: &mut Store,
        templates: &Templates,
        root: TermId,
        first: u32,
        scratch: &mut Scratch,
        rigid: &mut impl FnMut(&Store, ItemId) -> Result<bool, Error>,
    ) -> Result<FreshRegions, Error> {
        let mut next = first;
        let mut rebuilt_callables = false;
        let mut projections = false;
        scratch.work.push(Work::Enter(root, 0));
        while let Some(work) = scratch.work.pop() {
            self.step()?;
            match work {
                Work::Enter(id, depth) => {
                    let id = self.root(id)?;
                    // Reuse the freshening walk to see projected arguments, including
                    // closed canonical subtrees. Rebuilding may shift their binders.
                    projections |= match self.terms[id.0 as usize].value {
                        Value::Known(ty) => store.contains_projections(ty) == Some(true),
                        Value::Construct(Constructor::Projection(_)) => true,
                        _ => false,
                    };
                    if self.closed(store, id, depth)? || scratch.closed.contains(&(id, depth)) {
                        scratch.values.push(id);
                        continue;
                    }
                    let original = self.materialize(store, id)?;
                    if let Constructor::Adt(item, _) | Constructor::AdtArgs(item, _) = original
                        && rigid(store, item)?
                    {
                        scratch.closed.insert((id, depth));
                        scratch.values.push(id);
                        continue;
                    }
                    if matches!(original, Constructor::Projection(_)) {
                        if self.opaque_projection(id).is_some() {
                            // An opaque result has no exposed reference shape.
                            // Its operation inputs remain invariant identities.
                            scratch.values.push(id);
                            continue;
                        }
                        return Err(Error::Normalization);
                    }
                    if let Constructor::FunctionItemArgs(item, args) = original {
                        let term = self.terms[id.0 as usize];
                        scratch.work.push(Work::Item(
                            GenericItem {
                                item,
                                arguments: args,
                                depth,
                            },
                            scratch.values.len(),
                        ));
                        self.account_work((term.end - term.start) as usize)?;
                        scratch.work.extend(
                            self.edges[term.start as usize + 1..term.end as usize]
                                .iter()
                                .rev()
                                .map(|id| Work::Enter(*id, depth)),
                        );
                        continue;
                    }
                    if let Constructor::FunctionItem(item, args) = original {
                        scratch.arguments.clear();
                        for region in store.region_args(args).ok_or(Error::InvalidIdentity)? {
                            self.step()?;
                            scratch.arguments.push(fresh(*region, depth, &mut next)?);
                        }
                        let value = self.fresh_item(store, templates, item, &scratch.arguments)?;
                        rebuilt_callables = true;
                        projections |= store.contains_projections(
                            templates.get(item).ok_or(Error::InvalidIdentity)?.signature,
                        ) == Some(true);
                        scratch.values.push(value);
                        continue;
                    }
                    let term = self.terms[id.0 as usize];
                    let kind = match original {
                        Constructor::AdtArgs(item, args) => Constructor::AdtArgs(
                            item,
                            self.argument_regions(args, |r| fresh(r, depth, &mut next))?,
                        ),
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
                Work::Item(item, mark) => {
                    let term = self.fresh_generic_item(
                        store,
                        templates,
                        item,
                        &scratch.values[mark..],
                        &mut next,
                    )?;
                    rebuilt_callables = true;
                    projections |= store.contains_projections(
                        templates
                            .get(item.item)
                            .ok_or(Error::InvalidIdentity)?
                            .signature,
                    ) == Some(true);
                    scratch.values.truncate(mark);
                    scratch.values.push(term);
                }
            }
        }
        let result = scratch.values.pop().ok_or(Error::InvalidIdentity)?;
        if !scratch.values.is_empty() {
            return Err(Error::InvalidIdentity);
        }
        Ok(FreshRegions {
            term: result,
            regions: next.checked_sub(first).ok_or(Error::Overflow)?,
            needs_normalization: rebuilt_callables && projections,
        })
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
    fn fresh_generic_item(
        &mut self,
        store: &mut Store,
        templates: &Templates,
        item: GenericItem,
        children: &[TermId],
        next: &mut u32,
    ) -> Result<TermId, Error> {
        let GenericItem {
            item,
            arguments: args,
            depth,
        } = item;
        let template = templates.get(item).ok_or(Error::InvalidIdentity)?;
        let parameters = templates
            .generic_parameters(item)
            .ok_or(Error::InvalidIdentity)?;
        let mut types = children.iter();
        let mut arguments = Vec::new();
        let mut regions = Vec::new();
        let mut bindings = Vec::new();
        let count = self.slots(args)?.len();
        self.account_work(count)?;
        for index in 0..count {
            let argument = match self.slots(args)?[index] {
                Slot::Lifetime(region) => {
                    let region = fresh(region, depth, next)?;
                    regions.push(region);
                    Argument::Lifetime(region)
                }
                Slot::Type => Argument::Type(*types.next().ok_or(Error::InvalidIdentity)?),
                Slot::Const(id) => Argument::Const(id),
            };
            if !matches!(argument, Argument::Lifetime(_)) {
                let parameter = *parameters
                    .get(bindings.len())
                    .ok_or(Error::InvalidIdentity)?;
                bindings.push((parameter, argument));
            }
            arguments.push(argument);
        }
        if regions.len() != template.parameters as usize
            || bindings.len() != parameters.len()
            || types.next().is_some()
        {
            return Err(Error::InvalidIdentity);
        }
        let mut bound = Bound::new(regions);
        let signature = bound
            .apply(store, template.signature)
            .map_err(|_| Error::InvalidIdentity)?;
        self.account_work(bound.steps)?;
        let signature = self.instantiate(store, signature, &bindings)?;
        self.function_item(item, signature, &arguments)
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
