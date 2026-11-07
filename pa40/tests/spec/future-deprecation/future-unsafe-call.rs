#![feature(no_core,lang_items)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]trait PointeeSized{}
#[lang="meta_sized"]trait MetaSized:PointeeSized{}
#[lang="sized"]trait Sized:MetaSized{}
#[deprecated(since="TBD",note="retained reason")]unsafe fn dangerous()->u8{23}
#[deprecated(since="TBD",note="future")]pub fn old()->u8{dangerous()}
#[deny(deprecated)]pub fn use_old(){let _=old();}
