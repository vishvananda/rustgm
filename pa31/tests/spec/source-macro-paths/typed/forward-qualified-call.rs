#![feature(lang_items)]
#![feature(no_core)]
#![no_core]
#![allow(internal_features, unused_imports, unused_macros)]
crate::option_env!();
#[macro_export] macro_rules! option_env { () => {}; }

#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
