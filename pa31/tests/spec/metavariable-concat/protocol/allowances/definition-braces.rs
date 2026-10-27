#![feature(no_core, lang_items, rustc_attrs,allow_internal_unstable)]
#![no_core]
#![allow(internal_features)]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
#[allow_internal_unstable(macro_metavar_expr_concat)]macro_rules!make{($d:tt)=>{macro_rules!inner{()=>{fn $d {concat(foo,_x)}()->u8{7}}}}}make!($);inner!();fn use_it(){let _=foo_x();}
