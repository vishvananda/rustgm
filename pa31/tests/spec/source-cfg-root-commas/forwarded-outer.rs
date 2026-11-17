#![feature(no_core, lang_items, rustc_attrs)]
#![no_core]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
#[rustc_builtin_macro]
macro_rules! cfg { ($($tt:tt)*) => {}; }
macro_rules! q { ($m:meta) => { cfg!($m,) }; }
pub const OBSERVER: [(); 1] = [(); q!(all(unix,)) as usize];
