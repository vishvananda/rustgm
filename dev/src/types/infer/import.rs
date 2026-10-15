//! Copy only a query's reachable graph; failed probes cannot constrain the caller.
use super::{
    Constructor, Context, Error, Store, TermId, Value,
    arguments::{Argument, Slot},
    constants::ConstTermId,
};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Import {
    types: HashMap<TermId, TermId>,
    constants: HashMap<ConstTermId, ConstTermId>,
}
impl Context {
    /// Import a reachable graph from another context sharing the canonical store.
    /// Preserves repeated variables and const equivalence classes, but deliberately
    /// imports no region obligations or unrelated body constraints.
    /// # Errors
    /// Rejects invalid identities, compact overflow and exhausted work.
    pub fn import(
        &mut self,
        store: &Store,
        source: &mut Self,
        root: TermId,
    ) -> Result<TermId, Error> {
        let mut mappings = Import::default();
        self.import_with(store, source, root, &mut mappings)
    }
    pub(super) fn import_with(
        &mut self,
        store: &Store,
        source: &mut Self,
        root: TermId,
        mappings: &mut Import,
    ) -> Result<TermId, Error> {
        let root = source.root(root)?;
        let mut work = vec![(root, false)];
        let mut children = Vec::new();
        while let Some((id, finish)) = work.pop() {
            self.step()?;
            let id = source.root(id)?;
            if mappings.types.contains_key(&id) {
                continue;
            }
            let term = source.terms[id.0 as usize];
            let kind = match term.value {
                Value::Variable(domain) => {
                    mappings.types.insert(id, self.variable(domain)?);
                    continue;
                }
                Value::Known(ty) => {
                    mappings.types.insert(id, self.known(store, ty)?);
                    continue;
                }
                Value::Construct(kind) => kind,
            };
            if !finish {
                self.account_work((term.end - term.start) as usize)?;
                work.push((id, true));
                work.extend(
                    source.edges[term.start as usize..term.end as usize]
                        .iter()
                        .rev()
                        .map(|id| (*id, false)),
                );
                continue;
            }
            children.clear();
            for index in term.start..term.end {
                let id = source.root(source.edges[index as usize])?;
                children.push(*mappings.types.get(&id).ok_or(Error::InvalidIdentity)?);
            }
            let copied = mappings.construct(store, source, self, kind, &children)?;
            mappings.types.insert(id, copied);
        }
        mappings
            .types
            .get(&root)
            .copied()
            .ok_or(Error::InvalidIdentity)
    }
}

impl Import {
    pub(super) fn constant(
        &mut self,
        store: &Store,
        source: &mut Context,
        destination: &mut Context,
        id: ConstTermId,
    ) -> Result<ConstTermId, Error> {
        destination.step()?;
        let id = source.constants().root(id)?;
        if let Some(&copied) = self.constants.get(&id) {
            return Ok(copied);
        }
        let ty = source.constants().ty(id)?;
        let copied = if let Some(value) = source.constants().value(id)? {
            destination.const_value(ty, value)?
        } else {
            destination.const_variable(store, ty)?
        };
        self.constants.insert(id, copied);
        Ok(copied)
    }
    fn construct(
        &mut self,
        store: &Store,
        source: &mut Context,
        destination: &mut Context,
        kind: Constructor,
        children: &[TermId],
    ) -> Result<TermId, Error> {
        if let Constructor::ArrayInfer(id) = kind {
            let id = self.constant(store, source, destination, id)?;
            return destination.construct(Constructor::ArrayInfer(id), children);
        }
        let (item, args, function) = match kind {
            Constructor::AdtArgs(item, args) => (item, args, false),
            Constructor::FunctionItemArgs(item, args) => (item, args, true),
            _ => return destination.construct(kind, children),
        };
        let count = source.slots(args)?.len();
        destination.account_work(count)?;
        let mut arguments = Vec::with_capacity(count);
        let mut types = children[usize::from(function)..].iter();
        for index in 0..count {
            arguments.push(match source.slots(args)?[index] {
                Slot::Lifetime(region) => Argument::Lifetime(region),
                Slot::Type => Argument::Type(*types.next().ok_or(Error::InvalidIdentity)?),
                Slot::Const(id) => {
                    Argument::Const(self.constant(store, source, destination, id)?)
                }
            });
        }
        if function {
            destination.function_item(
                item,
                *children.first().ok_or(Error::InvalidIdentity)?,
                &arguments,
            )
        } else {
            destination.nominal(item, &arguments)
        }
    }
}
