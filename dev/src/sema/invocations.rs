//! Call-operator receiver choices are typed facts shared with body construction.
use crate::{support::call_traits::CallTrait, types::TypeId};

#[derive(Clone, Copy)]
pub(crate) struct Invocation {
    /// None denotes an ordinary function reached through reference dereferencing.
    pub kind: Option<CallTrait>,
    pub dereferences: u32,
    pub borrow: Option<TypeId>,
    pub receiver: TypeId,
}
