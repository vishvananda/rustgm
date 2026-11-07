#![feature(no_core,lang_items)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]trait PointeeSized{}
#[lang="meta_sized"]trait MetaSized:PointeeSized{}
#[lang="sized"]trait Sized:MetaSized{}
#[deprecated(since="1.98.1",note="retained reason")]pub fn old()->u8{23}
#[deny(deprecated)]pub fn use_old(){let _=old();}
