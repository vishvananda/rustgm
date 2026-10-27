#![feature(no_core,lang_items,macro_metavar_expr)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
macro_rules! checked{(7)=>{}}
macro_rules! subject{($x:ident)=>{checked!(${ignore($x)} 7);}}
subject!(a);
