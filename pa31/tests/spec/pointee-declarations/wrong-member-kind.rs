#![feature(no_core,lang_items,rustc_attrs)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang="sized"] pub trait Sized: MetaSized {}
#[lang="pointee_trait"] pub trait Pointee: PointeeSized {
    #[lang="metadata_type"] fn Metadata();
}
#[lang="dyn_metadata"] pub struct DynMetadata<Dyn: PointeeSized> { pointer: *const Dyn }
