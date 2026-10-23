#![feature(no_core,lang_items,intrinsics,rustc_attrs,const_trait_impl)]
#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
pub mod intrinsics{use crate::Sized;#[rustc_intrinsic]pub const fn needs_drop<T:?Sized>()->bool;}
pub const fn needs_drop<T: ?Sized>() -> bool {
    const { intrinsics::needs_drop::<T>() }
}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
struct D;impl Drop for D{fn drop(&mut self){}}const fn paid<T:Copy>()->bool{needs_drop::<T>()}
#[unsafe(no_mangle)]pub extern "C" fn observe()->u8{(paid::<D>()) as u8}
