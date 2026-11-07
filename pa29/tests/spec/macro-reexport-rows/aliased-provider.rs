#![feature(no_core, lang_items, rustc_attrs, decl_macro)]
#![no_core]
#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}

mod private {
    pub mod inner {
        #[rustc_builtin_macro]
        pub macro file() { /* compiler built-in */ }
    }
}
// `inner` is public only through this alias, so its macro has no public placement.
pub use private::inner;

pub fn base() -> u32 {
    40
}
