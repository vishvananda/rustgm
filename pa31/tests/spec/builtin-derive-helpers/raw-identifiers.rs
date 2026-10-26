#![feature(no_core,lang_items,rustc_attrs,decl_macro)]
#![no_core]
#![allow(dead_code,internal_features)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[rustc_builtin_macro(r#Default,attributes(r#default,r#type))]pub macro Default($item:item){}
