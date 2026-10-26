#![feature(no_core,lang_items,rustc_attrs,extern_types)]
#![no_core]
#![allow(internal_features,incomplete_features,dead_code)]
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
unsafe extern "C" { pub type Opaque; }
pub fn by_value(_:Opaque){}
