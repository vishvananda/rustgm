#![feature(no_core,rustc_attrs,prelude_import,lang_items)]
#![no_core]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang="sized"] pub trait Sized: MetaSized {}
#[rustc_builtin_macro]
#[macro_export]
macro_rules! cfg { ($($t:tt)*) => { /* compiler built-in */ }; }
mod prelude { pub use crate::cfg; }
#[prelude_import] use crate::prelude::*;
#[no_implicit_prelude]
mod consumer { pub const CHECK: bool = cfg!(unix); }
