#![feature(rustc_attrs)]
#![feature(no_core,lang_items)]
#![no_core]
#[rustc_builtin_macro]macro_rules!cfg{($($t:tt)*)=>{}}
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[cfg(any())] fn f(){let _=cfg!(ub_checks);}
fn main(){}
