#![feature(no_core, lang_items, rustc_attrs)]
#![no_core]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
pub mod panic {
    macro_rules! selected_2015 { (tag [nested] {tokens}) => { 15u32 }; }
    pub(crate) use selected_2015 as panic_2015;
    macro_rules! selected_2021 { (tag [nested] {tokens}) => { 21u32 }; }
    pub(crate) use selected_2021 as panic_2021;
}
#[rustc_builtin_macro(core_panic)]
macro_rules! route { ($($input:tt)*) => { 9u32 }; }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { route!(foreign [nested] {tokens}) }
