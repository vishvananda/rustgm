#![feature(no_core, lang_items, macro_metavar_expr_concat)]
#![no_core]
#![allow(internal_features)]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
macro_rules!m{()=>{fn ${concat("e","́")}()->u8{7}}}m!();fn use_it(){let _=é();}
