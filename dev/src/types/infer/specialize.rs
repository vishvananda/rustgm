//! Open source binders only where a partial destination supplies callable structure.
use super::{Constructor, Context, Error, Kind, Store, TermId, TypeId, Value};
use crate::types::{RegionId, arguments::Argument, specialize::Specialized, substitute::Bound};
use std::collections::{HashMap, HashSet};

type Key = (TypeId, TermId);
enum Work {
    Enter(Key),
    Finish(Key, Constructor, usize, u32),
}

impl Context {
    /// Inspect the current graph without resolving holes or interning types.
    /// Unknown inputs do not establish the presence of a callable binder.
    ///
    /// # Errors
    /// Rejects invalid identities and exhausted inference work.
    pub fn contains_late_callables(&mut self, store: &Store, root: TermId) -> Result<bool, Error> {
        let mut current = root;
        let mut work = Vec::new();
        let mut seen = HashSet::new();
        loop {
            self.step()?;
            let id = self.root(current)?;
            let term = self.terms[id.0 as usize];
            match term.value {
                Value::Known(ty) => {
                    if store
                        .contains_late_callables(ty)
                        .ok_or(Error::InvalidIdentity)?
                    {
                        return Ok(true);
                    }
                }
                Value::Construct(kind) if self.binder_regions(kind)?.is_some_and(|n| n != 0) => {
                    return Ok(true);
                }
                Value::Construct(_) if term.start != term.end && seen.insert(id) => {
                    self.account_work((term.end - term.start) as usize)?;
                    work.extend_from_slice(&self.edges[term.start as usize..term.end as usize]);
                }
                _ => {}
            }
            let Some(next) = work.pop() else {
                return Ok(false);
            };
            current = next;
        }
    }

    /// Infer holes inside a callable using free source-binder instances. A hole
    /// representing an entire higher-ranked type retains that type's binders.
    /// The original source still requires a separate directed subsumption proof.
    /// Captured caller universals must already have free aliases.
    ///
    /// # Errors
    /// Rejects escaping source lifetimes, invalid identities and exhausted work.
    pub fn specialize_shape(
        &mut self,
        store: &mut Store,
        actual: TypeId,
        expected: TermId,
        first: u32,
    ) -> Result<Specialized, Error> {
        if first == 0 || store.escaping_binders(actual) != Some(0) {
            return Err(Error::InvalidIdentity);
        }
        let before = self.steps;
        let mut next = first;
        let mut work = vec![Work::Enter((actual, expected))];
        let mut values = Vec::new();
        let mut unchanged = HashMap::new();
        let mut children = Vec::new();
        while let Some(task) = work.pop() {
            self.step()?;
            let Work::Enter((mut ty, expected)) = task else {
                let Work::Finish(key, kind, count, start) = task else {
                    return Err(Error::InvalidIdentity);
                };
                let base = values
                    .len()
                    .checked_sub(count)
                    .ok_or(Error::InvalidIdentity)?;
                let ty = self.canonical(store, kind, &values[base..])?;
                values.truncate(base);
                values.push(ty);
                // Fresh source binders belong to their structural occurrence.
                // Only a subtree that allocated none is reusable at siblings.
                if next == start {
                    unchanged.insert(key, ty);
                }
                continue;
            };
            let expected = self.root(expected)?;
            let key = (ty, expected);
            if let Some(&ty) = unchanged.get(&key) {
                values.push(ty);
                continue;
            }
            let shape = self.terms[expected.0 as usize];
            let Value::Construct(target) = shape.value else {
                values.push(ty);
                unchanged.insert(key, ty);
                continue;
            };
            if store.contains_late_callables(ty) == Some(false)
                || matches!(target, Constructor::Projection(_))
            {
                values.push(ty);
                unchanged.insert(key, ty);
                continue;
            }
            let source = self.known(store, ty)?;
            let source = self.materialize(store, source)?;
            if !parallel(source, target) {
                values.push(ty);
                unchanged.insert(key, ty);
                continue;
            }
            let start = next;
            if let Some(bound_regions) = self.binder_regions(source)?
                && bound_regions != 0
            {
                ty = self.open_shape_binder(store, ty, bound_regions, &mut next)?;
            }
            children.clear();
            source_children(store, ty, &mut children)?;
            self.account_work(children.len())?;
            if children.len() != (shape.end - shape.start) as usize {
                return Err(Error::Mismatch);
            }
            let term = self.known(store, ty)?;
            let kind = self.materialize(store, term)?;
            work.push(Work::Finish(key, kind, children.len(), start));
            let edges = &self.edges[shape.start as usize..shape.end as usize];
            work.extend(
                children
                    .iter()
                    .copied()
                    .zip(edges.iter().copied())
                    .rev()
                    .map(Work::Enter),
            );
        }
        let ty = values.pop().ok_or(Error::InvalidIdentity)?;
        if !values.is_empty() {
            return Err(Error::InvalidIdentity);
        }
        Ok(Specialized {
            ty,
            regions: next - first,
            steps: self.steps - before,
        })
    }

    fn open_shape_binder(
        &mut self,
        store: &mut Store,
        ty: TypeId,
        count: u32,
        next: &mut u32,
    ) -> Result<TypeId, Error> {
        self.account_work(count as usize)?;
        let end = next
            .checked_add(count)
            .filter(|end| *end < 0x8000_0000)
            .ok_or(Error::Overflow)?;
        let mut bound = Bound::new((*next..end).map(RegionId).collect());
        *next = end;
        let ty = if matches!(store.kind(ty), Some(Kind::Dynamic { .. })) {
            crate::types::dynamic::open(store, ty, &mut bound)
        } else {
            bound.signature(store, ty, 0)
        }
        .map_err(|_| Error::InvalidIdentity)?;
        self.account_work(bound.steps)?;
        Ok(ty)
    }
}

fn parallel(left: Constructor, right: Constructor) -> bool {
    let array = |kind| {
        matches!(
            kind,
            Constructor::Array(_) | Constructor::ArrayTerm(_) | Constructor::ArrayInfer(_)
        )
    };
    !matches!(
        left,
        Constructor::Projection(_)
            | Constructor::FunctionItem(..)
            | Constructor::FunctionItemArgs(..)
    ) && (std::mem::discriminant(&left) == std::mem::discriminant(&right)
        || (array(left) && array(right)))
}

fn source_children(store: &Store, ty: TypeId, children: &mut Vec<TypeId>) -> Result<(), Error> {
    match store.kind(ty).ok_or(Error::InvalidIdentity)? {
        Kind::Function {
            parameters, result, ..
        } => {
            children.extend_from_slice(store.list(parameters).ok_or(Error::InvalidIdentity)?);
            children.push(result);
        }
        Kind::Tuple(list) => {
            children.extend_from_slice(store.list(list).ok_or(Error::InvalidIdentity)?);
        }
        Kind::Array { element, .. }
        | Kind::ArrayTerm { element, .. }
        | Kind::Slice(element)
        | Kind::Reference {
            referent: element, ..
        }
        | Kind::Pointer {
            pointee: element, ..
        } => children.push(element),
        Kind::Adt(_, arguments) | Kind::Opaque(_, arguments) | Kind::Dynamic { arguments, .. } => {
            children.extend(
                store
                    .arguments(arguments)
                    .ok_or(Error::InvalidIdentity)?
                    .filter_map(|arg| {
                        if let Argument::Type(ty) = arg {
                            Some(ty)
                        } else {
                            None
                        }
                    }),
            );
        }
        _ => return Err(Error::InvalidIdentity),
    }
    Ok(())
}
