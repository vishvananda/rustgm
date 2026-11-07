#![feature(no_core, lang_items, rustc_attrs, decl_macro)]
#![no_core]
#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}

#[rustc_builtin_macro]
#[macro_export]
macro_rules! line { () => { /* compiler built-in */ } }
#[rustc_builtin_macro]
#[macro_export]
macro_rules! column { () => { /* compiler built-in */ } }
#[rustc_builtin_macro]
#[macro_export]
macro_rules! cfg { ($($cfg:tt)*) => { /* compiler built-in */ } }
#[rustc_builtin_macro]
#[macro_export]
macro_rules! stringify { ($($t:tt)*) => { /* compiler built-in */ } }
#[macro_export]
macro_rules! forty_two { () => { 42 } }

// Like `core::fmt::macros`: the definition's own module is private, so only
// its re-exports reach it.
mod hidden {
    #[rustc_builtin_macro]
    pub macro concat($($t:tt)*) { /* compiler built-in */ }
}
pub mod builtin {
    #[rustc_builtin_macro]
    pub macro file() { /* compiler built-in */ }
    pub use crate::hidden::concat;
}
// A second definition named `file`: another identity, the same expander.
pub mod alternate {
    #[rustc_builtin_macro]
    pub macro file() { /* compiler built-in */ }
}
// Root re-exports, like `pub use crate::macros::builtin::derive;`.
pub use builtin::file;
pub use crate::stringify as spelled;

// The shape of core's prelude: explicit re-exports, then a glob of them.
pub mod prelude {
    pub mod v1 {
        pub use crate::{cfg, column, forty_two, line, stringify};
        pub use crate::builtin::file;
        pub use crate::hidden::concat;
    }
    pub mod rust_2024 {
        pub use super::v1::*;
    }
}

pub fn base() -> u32 {
    40
}
