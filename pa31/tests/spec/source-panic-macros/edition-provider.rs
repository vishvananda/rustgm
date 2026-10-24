#![feature(no_core, lang_items, rustc_attrs, allow_internal_unstable)]
#![no_core]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
pub mod panic {
    #[macro_export]
    macro_rules! selected_2015 { (tag [nested] {tokens}) => { 15u32 }; }
    pub use selected_2015 as panic_2015;
    #[macro_export]
    macro_rules! selected_2021 { (tag [nested] {tokens}) => { 21u32 }; }
    pub use selected_2021 as panic_2021;
}
#[macro_export]
#[rustc_builtin_macro(core_panic)]
macro_rules! route { ($($input:tt)*) => { 9u32 }; }
#[macro_export]
macro_rules! ordinary_wrapper { ($($input:tt)*) => { $crate::route!($($input)*) }; }
#[allow_internal_unstable(edition_panic)]
#[macro_export]
macro_rules! edition_wrapper { ($($input:tt)*) => { $crate::route!($($input)*) }; }
