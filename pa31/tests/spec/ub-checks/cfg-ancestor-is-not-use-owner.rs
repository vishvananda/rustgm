#![feature(rustc_attrs)]
#![feature(no_core,lang_items)]
#![no_core]
#![feature(allow_internal_unstable)]
#[rustc_builtin_macro]macro_rules!cfg{($($t:tt)*)=>{}}
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
macro_rules! inner {()=>{cfg!(ub_checks)}}
#[allow_internal_unstable(cfg_ub_checks)]
macro_rules! outer {()=>{inner!()}}
fn main(){let _=outer!();}
