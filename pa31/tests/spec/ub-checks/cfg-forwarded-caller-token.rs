#![feature(rustc_attrs)]
#![feature(no_core,lang_items)]
#![no_core]
#![feature(allow_internal_unstable)]
#[rustc_builtin_macro]macro_rules!cfg{($($t:tt)*)=>{}}
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[allow_internal_unstable(cfg_ub_checks)]
macro_rules! m {($p:meta)=>{cfg!($p)}}
fn main(){let _=m!(any(all(),ub_checks));}
