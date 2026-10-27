#![feature(no_core, lang_items, macro_metavar_expr_concat)]
#![no_core]
#![allow(internal_features)]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
macro_rules!m{($n:literal)=>{fn ${concat(foo,$n)}()->u8{7}}}m!(9999999999999999999999999999999999999999999999999999999999999999);fn use_it(){let _=foo9999999999999999999999999999999999999999999999999999999999999999();}
