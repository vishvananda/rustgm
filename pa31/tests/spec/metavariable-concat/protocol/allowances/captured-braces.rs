#![feature(no_core, lang_items, rustc_attrs,allow_internal_unstable)]
#![no_core]
#![allow(internal_features)]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
#[allow_internal_unstable(macro_metavar_expr_concat)]macro_rules!make{($d:tt,$g:tt)=>{macro_rules!inner{()=>{fn $d $g()->u8{7}}}}}make!($,{concat(foo,_x)});inner!();fn use_it(){let _=foo_x();}
