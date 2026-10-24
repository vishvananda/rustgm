#![feature(no_core, lang_items, rustc_attrs, decl_macro)]
#![no_core]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
macro_rules! cfg_select { ($($tt:tt)*) => { 9u32 }; }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { cfg_select! { arbitrary tokens } }
