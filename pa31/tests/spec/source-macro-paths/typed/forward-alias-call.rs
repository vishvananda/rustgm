#![feature(lang_items)]
#![feature(no_core)]
#![no_core]
#![allow(internal_features, unused_imports, unused_macros)]
mod prelude { pub use crate::option_env as selected; selected!(); }
#[macro_export] macro_rules! option_env { () => {}; }

#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
