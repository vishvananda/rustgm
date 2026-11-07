#![feature(no_core,lang_items)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]trait PointeeSized{}
#[lang="meta_sized"]trait MetaSized:PointeeSized{}
#[lang="sized"]trait Sized:MetaSized{}
#[deprecated(since="TBD",note="future")]#[unsafe(no_mangle)]pub extern "C" fn future_return()->u8{23}
#[deny(deprecated)]#[unsafe(no_mangle)]pub extern "C" fn observed()->u8{future_return()}
