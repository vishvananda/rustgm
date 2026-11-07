#![feature(no_core, lang_items)]
#![no_core]
// A provider named `core` without core's prelude modules.
#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}
pub fn base() -> u32 {
    40
}
