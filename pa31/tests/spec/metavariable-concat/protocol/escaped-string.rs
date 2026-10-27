#![feature(no_core,lang_items,macro_metavar_expr_concat)]
#![no_core]
#![allow(internal_features)]
#[lang="pointee_sized"]trait PointeeSized{}
#[lang="meta_sized"]trait MetaSized:PointeeSized{}
#[lang="sized"]trait Sized:MetaSized{}
macro_rules!m{()=>{fn ${concat("f\u{006f}o",_bar)}(){}}}m!();
