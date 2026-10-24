#![feature(no_core, lang_items, rustc_attrs, decl_macro)]
#![no_core]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
#[rustc_builtin_macro]
pub macro cfg_select($($tt:tt)*) { /* compiler built-in */ }
type Selected = cfg_select! { all() => { u32 } _ => { let value = 9; } };
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { 7u32 }
