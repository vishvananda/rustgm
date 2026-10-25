#![feature(no_core,lang_items)]
#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[cfg_attr(any(all(),ub_checks),allow(dead_code))] fn f(){}
fn main(){}
