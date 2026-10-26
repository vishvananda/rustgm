#![feature(no_core,lang_items,rustc_attrs)]
#![no_core]
#![allow(dead_code,internal_features)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[macro_export]#[rustc_builtin_macro(line,attributes(helper))]macro_rules!line{()=>{}}
