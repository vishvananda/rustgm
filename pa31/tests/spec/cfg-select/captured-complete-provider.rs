#![feature(no_core, lang_items, rustc_attrs, decl_macro)]
#![no_core]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
#[lang = "clone"] pub trait Clone: Sized { #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"] pub trait Copy: Clone {}
#[lang = "legacy_receiver"] pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
impl Clone for u32 { fn clone(&self) -> u32 { *self } }
impl Copy for u32 {}
#[rustc_builtin_macro]
pub macro cfg_select($($tt:tt)*) { /* compiler built-in */ }
macro_rules! select { ($value:ident) => { cfg_select! { all() => { $value } _ => { missing_macro!() } } }; }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { let value = 7u32; select!(value) }
