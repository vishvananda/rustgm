#![feature(no_core,lang_items,macro_metavar_expr)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
macro_rules! checked{((0,2,(0,0,2,2)(1,0,2,2))(1,2,(0,1,1,2)))=>{}}
macro_rules! subject{($( [ $($x:ident),* ] ),*)=>{checked!($( (${index()},${len()},$((${index()},${index(1)},${len()},${len(1)}) ${ignore($x)})*) )*);}}
subject!([a,b],[c]);
