#![feature(no_core, lang_items, rustc_attrs, allow_internal_unstable)]
#![no_core]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
#[allow_internal_unstable(fixture_café)]
macro_rules! route { () => { 7u32 }; }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 { route!() }
