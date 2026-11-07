#![feature(no_core, lang_items, decl_macro)]
#![no_core]
#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}

pub macro twice($value:expr) {
    $value
}

pub fn base() -> u32 {
    40
}
