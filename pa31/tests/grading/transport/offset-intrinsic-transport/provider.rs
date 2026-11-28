#![feature(auto_traits, intrinsics, core_intrinsics, staged_api)]
#![unstable(feature="offset_transport", issue="none")]
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
#[lang="tuple_trait"]#[rustc_deny_explicit_impl]pub trait Tuple{}
#[rustc_const_unstable(feature="offset_transport",issue="none")]
#[lang="fn_once"]pub const trait FnOnce<Args:Tuple>{#[lang="fn_once_output"]type Output;extern "rust-call" fn call_once(self,args:Args)->Self::Output;}
#[rustc_const_unstable(feature="offset_transport",issue="none")]
#[lang="fn_mut"]pub const trait FnMut<Args:Tuple>:FnOnce<Args>{extern "rust-call" fn call_mut(&mut self,args:Args)->Self::Output;}
#[rustc_const_unstable(feature="offset_transport",issue="none")]
#[lang="fn"]pub const trait Fn<Args:Tuple>:[const]FnMut<Args>{extern "rust-call" fn call(&self,args:Args)->Self::Output;}



#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[rustc_nounwind]
#[unstable(feature = "core_intrinsics", issue = "none")]
#[rustc_const_unstable(feature = "core_intrinsics", issue = "none")]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
#[lang = "offset_of"]
pub const fn offset_of<T: PointeeSized>(variant: u32, field: u32) -> usize;
#[repr(C)]
pub struct Fields {
    pub first: u8,
    pub middle: u64,
    pub last: u16,
}

#[rustc_const_unstable(feature="offset_transport",issue="none")]
pub const fn field_offset<T: PointeeSized>(ordinal: u32) -> usize {
    offset_of::<T>(0, ordinal)
}
