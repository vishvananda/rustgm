#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(no_core,lang_items,intra_doc_pointers,strict_provenance_lints)]
#![no_core]
#[lang="pointee_sized"]trait PointeeSized{}#[lang="meta_sized"]trait MetaSized:PointeeSized{}#[lang="sized"]trait Sized:MetaSized{}
pub fn answer()->u8{7}
