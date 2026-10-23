//! Instances of one checked template share its substitution-invariant proof facts.
//!
//! `body::instantiate::materialize` copies the template's blocks, statements,
//! values, projections, temporaries and storage facts, then rewrites only
//! types and type-carrying operands: local, value and projection types, call
//! signatures, intrinsic and associated-constant instances, unsizing
//! conversions, const arguments, indexed array lengths and dereference regions
//! (`substitute`), deferred inline constants (`anonymous::operands`) and
//! normalized projections (`projections::normalize`). Demand then resolves
//! dispatch (`body::dispatch::resolve`), which rewrites the same kinds of
//! operands and records closure-call entries, recomputes promotions
//! (`body::promote::instance`) and reads constants (`body::constants::instance`).
//! None of these adds, removes or reorders a block, statement, value,
//! projection, temporary or storage exit, or changes a terminator successor, a
//! projection base or dereference kind, or any place operand.
//!
//! The CFG order walk reads only terminator successors. Storage availability
//! reads only those structural records and the promotion membership of each
//! materialized temporary, which instantiation can change and is therefore part
//! of the cache key. Neither reads a type. Both results are thus identical for
//! every instance of a template with the same promoted temporaries; they are
//! kept from a template's second instance on (single-instance templates retain
//! nothing) and reused by every later instance of the family.
//!
//! Copy and destructor behavior, region components and the instance contract,
//! which also carries the obligations of dispatch-selected implementations and
//! specialized callee premises, are inputs to ownership, initialization,
//! cleanup selection, regions and loans; every instance still proves those for
//! itself. Debug builds re-prove the shared facts for every instance and
//! reject any difference.
use super::Error;
use crate::{
    body::ir::{Body, BodyKey},
    borrow::{order::Order, storage},
    support::hash::HashMap,
};

#[cfg(test)]
thread_local! {
    /// Test-only: let every instance prove its structure itself, the full
    /// re-proof path that sharing must match exactly.
    pub(crate) static REPROVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Test-only: instance proofs that reused another instance's facts.
    pub(crate) static SHARED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Test-only: families proved under more than one promotion key.
    pub(crate) static SPLIT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
/// Borrowed substitution-invariant facts for one instance body.
#[derive(Clone, Copy)]
pub(crate) struct Shared<'a> {
    pub order: Option<&'a Order>,
    pub storage: &'a storage::Analysis,
}
/// How one body proof obtains the facts instances of a template share.
pub(crate) enum Use<'a, 'b> {
    /// Prove them for this body alone (ordinary bodies).
    Own,
    /// Prove them into a slot kept for later instances of the template.
    Keep(&'b mut Option<Proved>),
    /// Reuse the facts an earlier instance of the template proved.
    Shared(Shared<'a>),
}
/// The order walk and storage proof of one body.
pub(crate) struct Proved {
    pub order: Option<Order>,
    pub storage: storage::Analysis,
}
impl Proved {
    pub(crate) fn new(body: &Body) -> Result<Self, Error> {
        let order = Order::new(body);
        let storage = storage::check_ordered(body, order.as_ref()).map_err(|e| Error {
            node: Some(e.node),
            message: e.message.into(),
        })?;
        Ok(Self { order, storage })
    }
}
struct Structure {
    promoted: Promoted,
    proved: Proved,
}
/// One bit per materialized temporary: whether it is promoted. Bodies with at
/// most 64 temporaries, nearly all of them, need no allocation.
#[derive(PartialEq, Eq)]
pub(super) enum Promoted {
    Word(u64),
    Words(Box<[u64]>),
}
#[derive(Default)]
pub(super) struct Cache {
    /// Every template with a proved instance; facts are kept from its second
    /// instance on, so a template demanded once retains nothing.
    families: HashMap<BodyKey, Vec<Structure>>,
}
pub(super) enum Found<'a> {
    /// The template's first instance: prove the facts and drop them.
    First,
    /// No earlier instance kept facts for these promoted temporaries.
    Keep,
    Shared(Shared<'a>),
}
impl Cache {
    /// Facts already proved for an instance of `body`'s checked template with
    /// the same promoted temporaries. Debug builds re-prove and compare them.
    pub(super) fn find(&mut self, body: &Body, promoted: &Promoted) -> Result<Found<'_>, Error> {
        #[cfg(test)]
        if REPROVE.get() {
            return Ok(Found::First);
        }
        let family = match self.families.entry(body.key()) {
            std::collections::hash_map::Entry::Occupied(family) => family.into_mut(),
            std::collections::hash_map::Entry::Vacant(family) => {
                family.insert(Vec::new());
                return Ok(Found::First);
            }
        };
        let Some(structure) = family.iter().find(|s| s.promoted == *promoted) else {
            return Ok(Found::Keep);
        };
        if cfg!(debug_assertions) {
            let Proved { order, storage } = Proved::new(body)?;
            if structure.proved.order != order || structure.proved.storage != storage {
                return Err(Error {
                    node: Some(body.source.into()),
                    message: "shared instance structure differs from its re-proof".into(),
                });
            }
        }
        #[cfg(test)]
        SHARED.set(SHARED.get() + 1);
        Ok(Found::Shared(Shared {
            order: structure.proved.order.as_ref(),
            storage: &structure.proved.storage,
        }))
    }
    pub(super) fn insert(&mut self, body: &Body, promoted: Promoted, proved: Proved) {
        let family = self.families.entry(body.key()).or_default();
        #[cfg(test)]
        if family.len() == 1 {
            SPLIT.set(SPLIT.get() + 1);
        }
        family.push(Structure { promoted, proved });
    }
}
/// The storage input instantiation can change: promotion of each temporary.
pub(super) fn promoted(body: &Body) -> Promoted {
    let contains = |bit: usize| {
        body.promotions
            .as_ref()
            .is_some_and(|promotions| promotions.contains(body.temporaries[bit].value))
    };
    if body.temporaries.len() <= 64 {
        let mut word = 0;
        for bit in 0..body.temporaries.len() {
            if contains(bit) {
                word |= 1 << bit;
            }
        }
        return Promoted::Word(word);
    }
    let mut words = vec![0; body.temporaries.len().div_ceil(64)].into_boxed_slice();
    for bit in 0..body.temporaries.len() {
        if contains(bit) {
            words[bit / 64] |= 1 << (bit % 64);
        }
    }
    Promoted::Words(words)
}
