//! Declaration own bounds group caller assumptions by their first closed value.
//!
//! Two distinct closed values (a const value or a primitive type) at the first
//! non-lifetime argument of one trait never unify. An own bound with such a value
//! needs trial equations only against assumptions with that value or with an
//! open argument there, so a family of distinct own bounds stays linear.
use super::{Assumptions, Head};
use crate::support::hash::HashMap;
use crate::types::{
    ItemId, Kind, Store, TypeId,
    arguments::{Argument, ConstId, ConstKind},
    infer::{self, Context},
    parameters::predicates::Predicate,
};

#[cfg(test)]
#[path = "../../../../tests/invariants/own_bound_discriminants.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Value {
    Const(ConstId),
    Type(TypeId),
}

/// Built once for one declaration environment and charged per indexed assumption.
/// The caller keeps it beside the `Assumptions` it was built from.
#[derive(Default)]
pub(in crate::sema) struct Discriminants {
    built: bool,
    closed: HashMap<(ItemId, Head, Value), Vec<Predicate>>,
    open: HashMap<(ItemId, Head), Vec<Predicate>>,
}

/// The first non-lifetime trait argument when it is a closed value.
pub(super) fn value(store: &Store, predicate: Predicate) -> Result<Option<Value>, infer::Error> {
    let (id, _) = predicate
        .bound
        .trait_parts()
        .ok_or(infer::Error::InvalidIdentity)?;
    let application = store
        .trait_application(id)
        .ok_or(infer::Error::InvalidIdentity)?;
    let arguments = store
        .arguments(application.arguments)
        .ok_or(infer::Error::InvalidIdentity)?;
    let first = arguments
        .into_iter()
        .find(|argument| !matches!(argument, Argument::Lifetime(_)));
    Ok(match first {
        Some(Argument::Const(id)) => {
            matches!(store.const_kind(id), Some(ConstKind::Value(_))).then_some(Value::Const(id))
        }
        Some(Argument::Type(ty)) => {
            matches!(store.kind(ty), Some(Kind::Primitive(_))).then_some(Value::Type(ty))
        }
        Some(Argument::Lifetime(_)) | None => None,
    })
}

impl Discriminants {
    pub(super) fn build(
        &mut self,
        assumptions: &Assumptions,
        store: &Store,
        context: &mut Context,
    ) -> Result<(), infer::Error> {
        if self.built {
            return Ok(());
        }
        self.built = true;
        for (layer, _) in assumptions.index.iter().flat_map(|index| index.layers()) {
            for (&(item, head), predicates) in &layer.candidates {
                for &predicate in predicates {
                    context.account_work(1)?;
                    match value(store, predicate)? {
                        Some(value) => self
                            .closed
                            .entry((item, head, value))
                            .or_default()
                            .push(predicate),
                        None => self.open.entry((item, head)).or_default().push(predicate),
                    }
                }
            }
        }
        Ok(())
    }

    /// The assumptions that can unify with an own bound whose first value is `value`.
    pub(super) fn candidates(
        &self,
        item: ItemId,
        head: Head,
        value: Value,
    ) -> impl Iterator<Item = Predicate> + '_ {
        self.closed
            .get(&(item, head, value))
            .into_iter()
            .chain(self.open.get(&(item, head)))
            .flatten()
            .copied()
    }
}
