#![feature(lang_items)]
#![feature(no_core)]
#![no_core]
#![allow(internal_features, unused_imports, unused_macros)]
mod local { pub use crate::actual as option_env; }
local::option_env!();
#[macro_export] macro_rules! actual { () => {}; }

#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
