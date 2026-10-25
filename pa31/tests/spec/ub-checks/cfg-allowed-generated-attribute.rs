#![feature(no_core,lang_items)]
#![no_core]
#![feature(allow_internal_unstable)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[allow_internal_unstable(cfg_ub_checks)]
macro_rules! m {()=>{#[cfg(any(all(),ub_checks))] fn f(){}}}
m!();fn main(){}
