#![feature(no_core,lang_items,rustc_attrs,decl_macro)]
#![no_core]
#![feature(allow_internal_unstable)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[rustc_builtin_macro]pub macro cfg_select($($tt:tt)*){}
#[allow_internal_unstable(cfg_ub_checks)]macro_rules! m{($p:meta)=>{cfg_select!{$p=>{1u8}_=>{2u8}}}}
fn main(){let _=m!(ub_checks);}
