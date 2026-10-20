//! Immutable graph facts without allocating canonical types ahead of demand.
use super::{Constructor, Context, Error, Store, TermId, TypeId, Value, arguments::Slot};
use crate::types::arguments::Argument;
#[cfg(test)]
#[path = "../../../tests/invariants/completed_transport.rs"]
mod tests;

/// Only graphs without type/const holes or projections receive these facts.
#[derive(Clone, Copy, Default)]
pub(super) struct Completed {
    pub identity: Option<TypeId>,
    pub regions: bool,
    pub callables: bool,
}
impl Completed {
    pub fn canonical(store: &Store, ty: TypeId) -> Result<Option<Self>, Error> {
        if store
            .contains_projections(ty)
            .ok_or(Error::InvalidIdentity)?
        {
            return Ok(None);
        }
        Ok(Some(Self {
            identity: Some(ty),
            regions: store.contains_regions(ty).ok_or(Error::InvalidIdentity)?,
            callables: store.contains_callables(ty).ok_or(Error::InvalidIdentity)?,
        }))
    }
    fn include(&mut self, child: Self) {
        self.regions |= child.regions;
        self.callables |= child.callables;
    }
}

impl Context {
    pub(super) fn completed(
        &self,
        store: &Store,
        root: TermId,
    ) -> Result<Option<Completed>, Error> {
        let ty = match self.terms[root.0 as usize].value {
            Value::Known(ty) => ty,
            Value::Construct(Constructor::Primitive(p)) => TypeId::primitive(p),
            Value::Construct(Constructor::Never) => TypeId::NEVER,
            _ => {
                return Ok(self
                    .settled
                    .as_ref()
                    .and_then(|map| map.get(&root))
                    .copied());
            }
        };
        Completed::canonical(store, ty)
    }

    /// Called by existing postorder walks, never a separate recursive scan.
    /// A missing child summary means no certificate, even if later equations
    /// could finish that child. No canonical type or constant is interned here.
    pub(super) fn remember_structure(
        &mut self,
        store: &Store,
        root: TermId,
    ) -> Result<Option<Completed>, Error> {
        if let Some(completed) = self.completed(store, root)? {
            return Ok(Some(completed));
        }
        let term = self.terms[root.0 as usize];
        let Value::Construct(kind) = term.value else {
            return Ok(None);
        };
        let Some(mut completed) = self.constructor_facts(store, kind)? else {
            return Ok(None);
        };
        self.account_work((term.end - term.start) as usize)?;
        for index in term.start..term.end {
            let child = self.root(self.edges[index as usize])?;
            let Some(facts) = self.completed(store, child)? else {
                return Ok(None);
            };
            completed.include(facts);
        }
        self.settled
            .get_or_insert_with(Default::default)
            .insert(root, completed);
        Ok(Some(completed))
    }

    fn constructor_facts(
        &mut self,
        store: &Store,
        kind: Constructor,
    ) -> Result<Option<Completed>, Error> {
        let mut facts = Completed {
            regions: matches!(kind, Constructor::Reference { .. }),
            callables: matches!(kind, Constructor::Function { .. }),
            ..Completed::default()
        };
        match kind {
            kind if kind.normalizable(store) => return Ok(None),
            Constructor::ArrayInfer(id) if self.constants().value(id)?.is_none() => {
                return Ok(None);
            }
            Constructor::Adt(_, args) | Constructor::FunctionItem(_, args) => {
                for arg in store.arguments(args).ok_or(Error::InvalidIdentity)? {
                    self.step()?;
                    match arg {
                        Argument::Lifetime(_) => facts.regions = true,
                        Argument::Type(ty) => {
                            let Some(child) = Completed::canonical(store, ty)? else {
                                return Ok(None);
                            };
                            facts.include(child);
                        }
                        Argument::Const(_) => {}
                    }
                }
            }
            _ => {}
        }
        if let Some((args, _)) = kind.application() {
            let count = self.slots(args)?.len();
            self.account_work(count)?;
            for index in 0..count {
                let slot = self.slots(args)?[index];
                match slot {
                    Slot::Lifetime(_) => facts.regions = true,
                    Slot::Const(id) if self.constants().value(id)?.is_none() => return Ok(None),
                    Slot::Const(_) | Slot::Type => {}
                }
            }
        }
        Ok(Some(facts))
    }
}
