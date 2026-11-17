#![feature(no_core, lang_items, rustc_attrs, allow_internal_unstable, cfg_target_thread_local)]
#![no_core]
#![allow(internal_features, dead_code)]
#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "clone"]
pub trait Clone: Sized {
    #[lang = "clone_fn"]
    fn clone(&self) -> Self;
}
#[lang = "copy"]
pub trait Copy: Clone {}
impl Clone for u8 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u8 {}
#[rustc_builtin_macro]
#[macro_export]
macro_rules! include {
    ($($tokens:tt)*) => {};
}
pub mod prelude {
    pub mod rust_2015 {}
    pub mod rust_2018 {}
    pub mod rust_2021 {
        pub struct From2021;
    }
    pub mod rust_2024 {
        pub struct From2024;
    }
}

impl Clone for bool { fn clone(&self) -> Self { *self } }
impl Copy for bool {}
#[rustc_builtin_macro]
macro_rules! cfg { ($($tokens:tt)*) => {}; }
#[rustc_builtin_macro]
macro_rules! compile_error { ($($tokens:tt)*) => {}; }
fn main() {}

#[cfg(target_thread_local)] const SELECTED: u8 = 7;
#[cfg(not(target_thread_local))] compile_error!("wrong TLS target branch");
pub fn query() -> u8 { SELECTED }
