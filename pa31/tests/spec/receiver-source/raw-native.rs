#![feature(auto_traits)]
#![feature(const_trait_impl,const_destruct)]
#![feature(arbitrary_self_types_pointers)]
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

pub struct Node{value:u32}impl Node{pub fn read(self:*const Self)->u32{unsafe{(*self).value}}}
pub fn probe()->u32{let node=Node{value:23};let p=&node as *const Node;p.read()}
#[unsafe(no_mangle)]pub extern "C" fn raw_read()->u32{probe()}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]#[rustc_const_unstable(feature="const_destruct",issue="none")]pub const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
