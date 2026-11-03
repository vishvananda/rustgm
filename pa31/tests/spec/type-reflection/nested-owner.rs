#![feature(no_core,lang_items,rustc_attrs)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized:PointeeSized {}
#[lang="sized"] pub trait Sized:MetaSized {}
mod nested{#[lang="type_info"] pub struct Renamed{pub kind:u8}}
