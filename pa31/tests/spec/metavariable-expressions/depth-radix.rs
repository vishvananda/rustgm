#![feature(no_core,lang_items,macro_metavar_expr)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
macro_rules! checked{(3 ; (0,3)(1,3)(2,3))=>{}}
macro_rules! subject{($($x:ident),*)=>{checked!(${count($x,0b0)} ; $( ${ignore($x)} (${index(0x0)},${len(0o0)}) )*);}}
subject!(a,b,c);
