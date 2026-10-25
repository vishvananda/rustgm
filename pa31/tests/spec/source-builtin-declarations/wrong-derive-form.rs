#![feature(no_core, lang_items, rustc_attrs, decl_macro, allow_internal_unstable)]
#![no_core]
#[lang="pointee_sized"] trait PointeeSized {}
#[lang="meta_sized"] trait MetaSized: PointeeSized {}
#[lang="sized"] trait Sized: MetaSized {}
#[rustc_builtin_macro(From)] macro_rules! actual {()=>{9u8};}
