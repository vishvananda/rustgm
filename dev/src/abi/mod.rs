//! Target representation facts derived once from canonical semantic types.
pub mod classification;
pub mod layout;
pub mod metadata;
#[path = "representation_equivalence.rs"]
pub(crate) mod storage_relation;
pub mod tails;
pub mod templates;
