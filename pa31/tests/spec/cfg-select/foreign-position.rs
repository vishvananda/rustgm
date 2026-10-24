#![feature(no_core, lang_items, rustc_attrs, decl_macro)]
#![no_core]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
#[rustc_builtin_macro]
pub macro cfg_select($($tt:tt)*) { /* compiler built-in */ }
unsafe extern "C" { cfg_select! { all() => { fn abs(value: i32) -> i32; } _ => { fn unknown_external() -> u32; } } }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { unsafe { abs(7i32) as u32 } }
