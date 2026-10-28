#![feature(staged_api)]
#![stable(feature="base",since="1.0.0")]
#![allow(internal_features,dead_code)]
#![feature(lang_items,no_core,unboxed_closures)]
#![no_core]
#[stable(feature="base",since="1.0.0")]
#[lang="pointee_sized"]pub trait PointeeSized {}
#[stable(feature="base",since="1.0.0")]
#[lang="meta_sized"]pub trait MetaSized: PointeeSized {}
#[stable(feature="base",since="1.0.0")]
#[lang="sized"]pub trait Sized: MetaSized {}
#[stable(feature="base",since="1.0.0")]
trait Tr {}
#[expect(ineffective_unstable_trait_impl)]
#[unstable(feature="sample",issue="none")]
impl Tr for extern "rust-call" fn(()) {}
