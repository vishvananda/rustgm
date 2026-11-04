#![feature(no_core,lang_items,staged_api)]
#![no_core]
#![unstable(feature="inventory_api",issue="none")]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang="sized"] pub trait Sized: MetaSized {}
pub fn value()->u8{7u8}
