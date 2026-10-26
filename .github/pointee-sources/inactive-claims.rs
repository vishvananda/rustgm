#![feature(no_core,lang_items,rustc_attrs)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang="sized"] pub trait Sized: MetaSized {}
#[cfg(disabled)] #[lang="pointee_trait"] fn wrong() {}
#[cfg(disabled)] #[lang="metadata_type"] fn also_wrong() {}
#[cfg(disabled)] #[lang="dyn_metadata"] fn third_wrong() {}
