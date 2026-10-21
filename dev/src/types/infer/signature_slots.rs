//! A fresh public signature slot keeps its defining binder across assigned aliases.
mod classes;
mod transport;
use super::{Constructor, Context, Domain, Error, Store, TermId, Value};
use crate::types::closure::{ClosureId, Declaration};
use classes::Classes;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Class {
    Empty,
    Own(ClosureId),
    Mixed,
}
impl Class {
    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Empty, value) | (value, Self::Empty) => value,
            (Self::Own(a), Self::Own(b)) if a == b => self,
            _ => Self::Mixed,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Owner {
    pub origin: ClosureId,
    pub closed: bool,
}

pub(super) struct Slots {
    definitions: HashMap<TermId, Owner>,
    frames: HashMap<TermId, Owner>,
    classes: Classes,
}
impl Slots {
    fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            frames: HashMap::new(),
            classes: Classes::default(),
        }
    }
    fn bounded(&self) -> Result<(), Error> {
        let slots = self.definitions.len() + self.frames.len() + self.classes.len();
        // Conservative logical charge for three sparse map entries, not allocator telemetry.
        if slots > (64 * 1024 * 1024) / 128 {
            return Err(Error::Budget);
        }
        Ok(())
    }
}

impl Context {
    /// Only the original pending closure signature can define fresh public holes.
    /// Imports retain origin and loss history but cannot acquire this authority.
    pub(crate) fn define_signature_slots(
        &mut self,
        store: &Store,
        closure: TermId,
        signature: TermId,
        slots: &[TermId],
    ) -> Result<(), Error> {
        let Some(Constructor::Closure(origin, _)) = self.head(store, closure)? else {
            return Err(Error::InvalidIdentity);
        };
        if store.closure_declaration(origin) != Some(Declaration::Pending)
            || self.child(store, closure, 0)? != signature
            || self.root(signature)? != signature
        {
            return Err(Error::InvalidIdentity);
        }
        let Some(Constructor::Function {
            parameters,
            bound_regions,
            ..
        }) = self.head(store, signature)?
        else {
            return Err(Error::InvalidIdentity);
        };
        if slots.len() != parameters as usize + 1 {
            return Err(Error::InvalidIdentity);
        }
        // An empty public binder has no local slot that needs assigned-hole transport.
        if bound_regions == 0 {
            return Ok(());
        }
        self.account_work(slots.len())?;
        let mut distinct = std::collections::HashSet::new();
        for (index, &slot) in slots.iter().enumerate() {
            let term = self
                .terms
                .get(slot.0 as usize)
                .ok_or(Error::InvalidIdentity)?;
            if !distinct.insert(slot)
                || term.parent != slot
                || term.rank != 0
                || !matches!(term.value, Value::Variable(Domain::Any))
                || self.child(store, signature, index)? != slot
                || self.signature_slots.as_ref().is_some_and(|facts| {
                    facts.definitions.contains_key(&slot) || facts.classes.contains_key(slot)
                })
            {
                return Err(Error::InvalidIdentity);
            }
        }
        let facts = self
            .signature_slots
            .get_or_insert_with(|| Box::new(Slots::new()));
        if facts.frames.contains_key(&signature) {
            return Err(Error::InvalidIdentity);
        }
        let owner = Owner {
            origin,
            closed: true,
        };
        facts.frames.insert(signature, owner);
        for &slot in slots {
            facts.definitions.insert(slot, owner);
            facts.classes.fresh(slot, Class::Own(origin))?;
        }
        facts.bounded()
    }

    fn slot_class(&mut self, root: TermId) -> Result<Class, Error> {
        let variable = self
            .terms
            .get(root.0 as usize)
            .ok_or(Error::InvalidIdentity)?
            .was_variable;
        let fallback = if variable { Class::Mixed } else { Class::Empty };
        let Some(facts) = &mut self.signature_slots else {
            return Ok(fallback);
        };
        let (class, steps) = facts.classes.class(root, fallback)?;
        self.account_work(steps)?;
        Ok(class)
    }

    /// Persistent equations join histories even when assigned roots need no link.
    /// Alpha comparison of separate closed function frames never calls this hook.
    pub(super) fn join_slot_classes(&mut self, a: TermId, b: TermId) -> Result<(), Error> {
        self.merge_slot_classes(a, b, false)
    }
    pub(super) fn link_slot_classes(&mut self, a: TermId, b: TermId) -> Result<(), Error> {
        self.merge_slot_classes(a, b, true)
    }
    fn merge_slot_classes(&mut self, a: TermId, b: TermId, linked: bool) -> Result<(), Error> {
        let Some(facts) = &self.signature_slots else {
            return Ok(());
        };
        if !self
            .terms
            .get(a.0 as usize)
            .ok_or(Error::InvalidIdentity)?
            .was_variable
            || !self
                .terms
                .get(b.0 as usize)
                .ok_or(Error::InvalidIdentity)?
                .was_variable
        {
            return Ok(());
        }
        if !linked && !facts.classes.contains_key(a) && !facts.classes.contains_key(b) {
            return Ok(());
        }
        self.step()?;
        let defaults = (self.slot_class(a)?, self.slot_class(b)?);
        let facts = self
            .signature_slots
            .as_mut()
            .ok_or(Error::InvalidIdentity)?;
        let steps = facts.classes.unite(a, b, defaults)?;
        facts.bounded()?;
        self.account_work(steps)
    }

    pub(super) fn signature_frame(&self, root: TermId) -> Option<Owner> {
        self.signature_slots.as_ref()?.frames.get(&root).copied()
    }
    pub(super) fn signature_definition(
        &mut self,
        original: TermId,
        _root: TermId,
    ) -> Result<Option<(Owner, bool)>, Error> {
        let Some(owner) = self
            .signature_slots
            .as_ref()
            .and_then(|facts| facts.definitions.get(&original))
            .copied()
        else {
            return Ok(None);
        };
        Ok(Some((
            owner,
            self.slot_class(original)? == Class::Own(owner.origin),
        )))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/signature_owner.rs"]
mod tests;
