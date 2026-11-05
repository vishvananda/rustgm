#![feature(arbitrary_self_types)]
#![feature(no_core,lang_items,rustc_attrs,staged_api)]
#![no_core]
#![unstable(feature="receiver_fixture",issue="none")]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u32{fn clone(&self)->Self{*self}}impl Copy for u32{}
impl<T:PointeeSized> Clone for *const T{fn clone(&self)->Self{*self}}impl<T:PointeeSized> Copy for *const T{}
impl<T:PointeeSized> Clone for *mut T{fn clone(&self)->Self{*self}}impl<T:PointeeSized> Copy for *mut T{}
#[lang="deref"]pub trait Deref:PointeeSized{#[lang="deref_target"]type Target:?Sized;fn deref(&self)->&Self::Target;}
impl<T:?Sized> Deref for &T{type Target=T;fn deref(&self)->&T{*self}}
impl<T:?Sized> Deref for &mut T{type Target=T;fn deref(&self)->&T{self}}
#[lang = "receiver"]
#[unstable(feature = "arbitrary_self_types", issue = "44874")]
pub trait Receiver: PointeeSized {
    /// The target type on which the method may be called.
    #[rustc_diagnostic_item = "receiver_target"]
    #[lang = "receiver_target"]
    #[unstable(feature = "arbitrary_self_types", issue = "44874")]
    type Target: ?Sized;
}

#[unstable(feature = "arbitrary_self_types", issue = "44874")]
impl<P: ?Sized, T: ?Sized> Receiver for P
where
    P: Deref<Target = T>,
{
    type Target = T;
}

pub struct Node{value:u32}
pub struct Pointer<T:?Sized>{pointer:*const T}
impl<T:?Sized> Receiver for Pointer<T>{type Target=T;}
impl Node{pub fn read(self:Pointer<Self>)->u32{unsafe{(*self.pointer).value}}}
pub fn probe()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}

pub fn call_0()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_1()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_2()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_3()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_4()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_5()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_6()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_7()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_8()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_9()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_10()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_11()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_12()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_13()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_14()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_15()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_16()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_17()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_18()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_19()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_20()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_21()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_22()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_23()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_24()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_25()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_26()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_27()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_28()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_29()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_30()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
pub fn call_31()->u32{let node=Node{value:23};Pointer{pointer:&node}.read()}
