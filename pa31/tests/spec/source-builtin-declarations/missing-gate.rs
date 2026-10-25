#![feature(no_core, lang_items, decl_macro, allow_internal_unstable)]
#![no_core]
#[lang="pointee_sized"] trait PointeeSized {}
#[lang="meta_sized"] trait MetaSized: PointeeSized {}
#[lang="sized"] trait Sized: MetaSized {}
#[rustc_builtin_macro(offload_kernel)]pub macro actual($item:item){}
