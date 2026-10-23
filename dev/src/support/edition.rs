//! Defining language semantics, shared by source parsing and imported bodies.
//! This identity is not a metadata wire tag or a claim of driver support.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Edition {
    Rust2021,
    #[default]
    Rust2024,
}
