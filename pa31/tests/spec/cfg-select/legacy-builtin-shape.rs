#![feature(no_core, lang_items, rustc_attrs, decl_macro)]
#![no_core]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
#[rustc_builtin_macro]
macro_rules! cfg_select { ($($tt:tt)*) => {}; }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { cfg_select! { all() => { 7u32 } _ => { 9u32 } } }
