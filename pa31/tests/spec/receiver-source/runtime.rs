#![feature(const_trait_impl,const_destruct)]
#![feature(auto_traits)]
#![feature(intrinsics)]
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
pub struct Shell<T>(T);impl<T> Receiver for Shell<T>{type Target=T;}
impl Node{pub fn shared(self:&Pointer<Self>)->u32{unsafe{(*self.pointer).value}}pub fn nested(self:Shell<Pointer<Self>>)->u32{unsafe{(*self.0.pointer).value}}}
#[unsafe(no_mangle)]pub extern "C" fn direct()->u32{let node=Node{value:23};Node::read(Pointer{pointer:&node})}
#[unsafe(no_mangle)]pub extern "C" fn dot()->u32{let node=Node{value:17};Pointer{pointer:&node}.read()}
#[unsafe(no_mangle)]pub extern "C" fn shared()->u32{let node=Node{value:11};Pointer{pointer:&node}.shared()}
#[unsafe(no_mangle)]pub extern "C" fn nested()->u32{let node=Node{value:29};Shell(Pointer{pointer:&node}).nested()}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[rustc_intrinsic]pub const fn wrapping_add<T:Copy>(left:T,right:T)->T;
static mut CALLS:u32=0;fn make(pointer:*const Node)->Pointer<Node>{unsafe{CALLS=wrapping_add(CALLS,1);}Pointer{pointer}}
#[unsafe(no_mangle)]pub extern "C" fn once()->u32{unsafe{CALLS=0;}let node=Node{value:31};let value=make(&node).read();unsafe{wrapping_add(value,CALLS)}}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]#[rustc_const_unstable(feature="const_destruct",issue="none")]pub const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
