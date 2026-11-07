#![feature(no_core,lang_items,staged_api)]
#![no_core]
#![stable(feature="deprecation_packet",since="1.0.0")]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]trait PointeeSized{}
#[lang="meta_sized"]trait MetaSized:PointeeSized{}
#[lang="sized"]trait Sized:MetaSized{}
#[stable(feature="deprecation_packet",since="1.0.0")]#[deprecated(since="soon",note="retained reason")]pub fn old()->u8{23}
#[stable(feature="deprecation_packet",since="1.0.0")]#[allow(deprecated)]pub fn use_old(){let _=old();}
