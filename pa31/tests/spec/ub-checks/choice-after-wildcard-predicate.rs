#![feature(no_core,lang_items,rustc_attrs,decl_macro)]
#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[rustc_builtin_macro]pub macro cfg_select($($tt:tt)*){}
fn main(){let _=cfg_select!{_=>{1u8} ub_checks=>{2u8}};}
