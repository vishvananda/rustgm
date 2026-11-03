#![feature(no_core,lang_items,rustc_attrs)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized:PointeeSized {}
#[lang="sized"] pub trait Sized:MetaSized {}
#[lang="type_info"] pub struct Reflected {pub kind:u8}
#[lang="type_id"] pub struct Identity {pub data:[*const ();2]}
#[unsafe(no_mangle)] pub extern "C" fn observed_kind()->u8{let value=Reflected{kind:7};value.kind}
