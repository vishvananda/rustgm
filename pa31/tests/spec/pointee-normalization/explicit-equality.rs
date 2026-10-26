#![feature(no_core,lang_items,rustc_attrs)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang="sized"] pub trait Sized: MetaSized {}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="pointee_trait"] pub trait Pointee: PointeeSized {
    #[lang="metadata_type"] type Metadata: Sized;
}
#[lang="dyn_metadata"] pub struct DynMetadata<Dyn: PointeeSized> { pointer: *const Dyn }
pub fn stated_equality<T:Sized+Pointee<Metadata=usize>>(value:<T as Pointee>::Metadata)->usize{value}
