#![feature(no_core,lang_items,rustc_attrs,staged_api)]
#![no_core]
#![stable(feature="root",since="1.0.0")]
#![allow(internal_features,dead_code,unused_variables)]
#[stable(feature="root",since="1.0.0")]#[lang="pointee_sized"]pub trait PointeeSized{}
#[stable(feature="root",since="1.0.0")]#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[stable(feature="root",since="1.0.0")]#[lang="sized"]pub trait Sized:MetaSized{}
#[stable(feature="root",since="1.0.0")]#[rustc_const_unstable(feature="door",issue="none")]pub const fn target()->u8{7}
#[rustc_allow_const_fn_unstable(other)]
#[rustc_allow_const_fn_unstable(door)]
#[stable(feature="root",since="1.0.0")]#[rustc_const_stable(feature="stable_call",since="1.0.0")]pub const fn caller()->u8{target()}
