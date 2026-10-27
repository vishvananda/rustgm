#![feature(no_core, lang_items, macro_metavar_expr_concat,decl_macro)]
#![no_core]
#![allow(internal_features)]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
pub macro m($name:ident){{fn ${concat($name,_x)}()->u8{7} ${concat($name,_x)}()}}fn use_it(){let _=m!(foo);}
