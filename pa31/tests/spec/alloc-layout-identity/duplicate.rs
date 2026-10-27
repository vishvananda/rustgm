#![feature(no_core,lang_items)]#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="alloc_layout"]pub struct One;#[lang="alloc_layout"]pub struct Two;
