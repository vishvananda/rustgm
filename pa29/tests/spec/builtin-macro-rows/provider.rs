#![feature(no_core, lang_items, rustc_attrs, decl_macro)]
#![no_core]
#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}

#[rustc_builtin_macro]
#[macro_export]
macro_rules! line { () => { /* compiler built-in */ } }
#[rustc_builtin_macro]
#[macro_export]
macro_rules! column { () => { /* compiler built-in */ } }
#[rustc_builtin_macro]
#[macro_export]
macro_rules! cfg { ($($cfg:tt)*) => { /* compiler built-in */ } }
#[rustc_builtin_macro]
#[macro_export]
macro_rules! stringify { ($($t:tt)*) => { /* compiler built-in */ } }
#[rustc_builtin_macro]
#[macro_export]
macro_rules! compile_error { ($msg:expr $(,)?) => { /* compiler built-in */ } }

pub mod builtin {
    #[rustc_builtin_macro]
    pub macro concat($($t:tt)*) { /* compiler built-in */ }
    #[rustc_builtin_macro]
    pub macro file() { /* compiler built-in */ }
    pub mod deeper {
        #[rustc_builtin_macro]
        pub macro module_path() { /* compiler built-in */ }
        #[rustc_builtin_macro]
        pub macro test($item:item) { /* compiler built-in */ }
    }
}

pub fn base() -> u32 {
    40
}
