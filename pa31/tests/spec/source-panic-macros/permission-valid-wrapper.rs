#![feature(no_core, lang_items, rustc_attrs, allow_internal_unstable)]
#![no_core]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
pub mod panic {
    macro_rules! chosen { (tag [nested] {tokens}) => { 21u32 }; }
    pub(crate) use chosen as panic_2021;
}
#[rustc_builtin_macro(core_panic)]
macro_rules! route { ($($input:tt)*) => { 9u32 }; }
#[allow_internal_unstable(edition_panic)]
macro_rules! wrapper { ($($input:tt)*) => { route!($($input)*) }; }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { wrapper!(tag [nested] {tokens}) }
