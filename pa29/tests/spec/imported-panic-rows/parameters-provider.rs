#![feature(no_core, lang_items, rustc_attrs, decl_macro)]
#![no_core]
#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}

// The single-rule form, as core's `const_error` is written.
#[rustc_macro_transparency = "semiopaque"]
pub macro twice($value:expr) {
    $value
}
