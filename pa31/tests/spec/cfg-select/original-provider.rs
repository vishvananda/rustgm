#![feature(no_core, lang_items, rustc_attrs, decl_macro)]
#![no_core]
#![feature(staged_api)]
#![stable(feature="cfg_select_fixture",since="1.0.0")]
#[stable(feature="cfg_select_fixture",since="1.0.0")]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[stable(feature="cfg_select_fixture",since="1.0.0")]
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[stable(feature="cfg_select_fixture",since="1.0.0")]
#[lang = "sized"] pub trait Sized: MetaSized {}
#[stable(feature = "cfg_select", since = "1.95.0")]
#[doc(alias = "cfg_if", alias = "cfg-if")]
#[rustc_diagnostic_item = "cfg_select"]
#[rustc_builtin_macro]
pub macro cfg_select($($tt:tt)*) {
    /* compiler built-in */
}
#[stable(feature="cfg_select_fixture",since="1.0.0")]
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { cfg_select! { all() => { 7u32 } _ => { 9u32 } } }
