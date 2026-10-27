#![feature(no_core, lang_items, macro_metavar_expr_concat)]
#![no_core]
#![allow(internal_features)]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
macro_rules! define {
    ($name:ident) => { pub fn ${concat($name, _small)}() -> u8 { 7 } };
}
define!(fmt);
fn main() { let _ = fmt_small(); }
