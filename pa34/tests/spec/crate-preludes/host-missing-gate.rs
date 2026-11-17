#![feature(no_core, lang_items, rustc_attrs, decl_macro, auto_traits)]
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

#[lang = "freeze"]
pub unsafe auto trait Freeze {}

pub mod provider {
    pub struct HostOnly;
    pub struct Same;
    pub trait Read { fn read(&self)->u8; }
    impl Read for Same { fn read(&self)->u8 { 7 } }
}
pub mod prelude {
    pub mod rust_2021 { pub use crate::{Clone,Copy,Sized}; pub struct From2021; }
    pub mod rust_2024 { pub use crate::{Clone,Copy,Sized}; pub use crate::provider::{HostOnly,Same,Read}; }
}
#[prelude_import]
use crate::prelude::rust_2024::*;
pub fn own()->crate::provider::HostOnly { HostOnly }
pub fn own_method()->u8 { Same.read() }
pub macro opaque($module:ident,$run:ident) {
    pub mod $module { pub fn $run()->Same { let value=Same; let observed=value.read(); value } }
}
pub macro captured($ty:ident,$value:ident,$method:ident,$module:ident,$run:ident) {
    pub mod $module { pub fn $run()->$ty { let value=$value; let observed=value.$method(); value } }
}
#[macro_export]
macro_rules! mixed_site {
    ($module:ident,$run:ident) => { pub mod $module { pub fn $run()->Same { let value=Same; let observed=value.read(); value } } };
}
