//! `RustGM`'s maintainer frontend. Text formats are views of typed phases.
// Shared source-loading helpers also serve library invariant tests.
#[cfg(test)]
extern crate self as rustgm;
pub mod abi;
pub mod body;
pub mod borrow;
pub mod const_eval;
pub mod expand;
pub mod lex;
pub mod literal;
pub mod lower;
pub mod lowir;
pub mod pipeline;
pub mod resolve;
#[cfg(test)]
#[path = "../tests/helpers/qualify.rs"]
mod rustc;
pub mod sema;
pub mod support;
pub mod syntax;
pub mod types;

#[cfg(test)]
#[path = "../tests/helpers/resolve.rs"]
mod test_source;
#[cfg(test)]
#[path = "../tests/helpers/held_sessions.rs"]
mod unpaid_cases;

#[cfg(test)]
#[path = "../tests/helpers/locations.rs"]
mod test_sites;

#[cfg(test)]
#[path = "../tests/provider_traits/cases.rs"]
pub(crate) mod provider_cases;
