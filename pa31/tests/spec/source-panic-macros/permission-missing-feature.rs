#![feature(no_core, lang_items, rustc_attrs)]
#![no_core]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
#[allow_internal_unstable(edition_panic)]
macro_rules! helper { () => { 7u32 }; }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { helper!() }
