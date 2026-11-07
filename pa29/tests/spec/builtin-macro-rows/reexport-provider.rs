#![feature(no_core, lang_items, rustc_attrs, decl_macro)]
#![no_core]
#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}

pub mod builtin {
    #[rustc_builtin_macro]
    pub macro concat($($t:tt)*) { /* compiler built-in */ }
}
// The shape of core's prelude: a public re-export of a placed builtin.
pub use builtin::concat;

pub fn base() -> u32 {
    40
}
