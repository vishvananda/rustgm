#![feature(auto_traits, negative_impls)]
// Authored Option, Result, and Ordering declarations with the pinned role shapes.
#![feature(try_blocks)]
#![feature(unboxed_closures)]
#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[rustc_diagnostic_item = "unwind_safe_trait"]
pub auto trait Safe {}

#[rustc_diagnostic_item = "ref_unwind_safe_trait"]
pub auto trait SharedSafe {}

impl<T: PointeeSized> !Safe for &mut T {}
impl<T: PointeeSized + SharedSafe> Safe for &T {}

pub struct Shared(pub u8);

pub fn require<T: PointeeSized + Safe>(_: &T) -> u8 {
    7
}

pub fn require_shared<T: PointeeSized + SharedSafe>(_: &T) -> u8 {
    11
}
