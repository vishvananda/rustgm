#![feature(no_core,rustc_attrs)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang="sized"] pub trait Sized: MetaSized {}
#[lang="pointee_trait"] pub trait Pointee: PointeeSized {
    #[lang="metadata_type"] type Metadata: Sized;
}
#[lang="dyn_metadata"] pub struct DynMetadata<Dyn: PointeeSized> { pointer: *const Dyn }
