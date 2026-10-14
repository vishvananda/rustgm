//! Built-in obligations have typed subjects; their spellings are presentation only.
use crate::types::TypeId;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Bound {
    Copy,
    Clone,
    Sized,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Predicate {
    pub ty: TypeId,
    pub bound: Bound,
}
