#![feature(no_core,lang_items,rustc_attrs,staged_api,const_trait_impl)]
#![no_core]
#![stable(feature="root",since="1.0.0")]
#![allow(internal_features,dead_code,unused_variables)]
#[stable(feature="root",since="1.0.0")]#[lang="pointee_sized"]pub trait PointeeSized{}
#[stable(feature="root",since="1.0.0")]#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[stable(feature="root",since="1.0.0")]#[lang="sized"]pub trait Sized:MetaSized{}
#[stable(feature="root",since="1.0.0")]#[rustc_const_stable(feature="method_const",since="1.0.0")]pub const trait Check{#[rustc_allow_const_fn_unstable(door)]
#[stable(feature="root",since="1.0.0")]fn allowed()->u8{7}}
