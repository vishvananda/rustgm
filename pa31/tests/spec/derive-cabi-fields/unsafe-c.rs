#![feature(no_core, lang_items, rustc_attrs, decl_macro)]
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
pub mod clone { pub use crate::Clone; }
#[rustc_builtin_macro]
pub macro derive($item:item) {}
#[rustc_builtin_macro]
pub macro Clone($item:item) {}
const SOURCE_POOL_PREFIX: &str = "source\x5fdecoded_prefix";
#[derive(Clone)]
pub enum Option<T> { None, Some(T) }
#[derive(Clone)]
pub struct Holder(pub Option<unsafe extern "C" fn()>) where Option<unsafe extern "C" fn()>: Clone;
pub fn cloned(value: &Holder) -> Holder { Clone::clone(value) }
