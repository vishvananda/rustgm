// Authored no_core support cutout: actual language-trait interfaces and primitive impl bodies.
// The complete original intrinsic function below is unchanged from pinned core.
#![feature(no_core,lang_items,intrinsics,rustc_attrs,register_tool,const_trait_impl,auto_traits)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
const impl<T:PointeeSized>Clone for *const T{fn clone(&self)->Self{*self}}
impl<T:PointeeSized>Copy for *const T{}
#[lang="not"]pub const trait Not{type Output;fn not(self)->Self::Output;}
const impl Not for bool{type Output=bool;fn not(self)->bool{!self}}
#[lang="eq"]pub const trait PartialEq<Rhs:PointeeSized=Self>:PointeeSized{fn eq(&self,other:&Rhs)->bool;fn ne(&self,other:&Rhs)->bool{!self.eq(other)}}
impl<T:PointeeSized>PartialEq for *const T{fn eq(&self,other:&*const T)->bool{*self==*other}}
#[lang="sync"]pub unsafe auto trait Sync{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]pub(crate) const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}

fn ordinary()->u8{99}
#[rustc_intrinsic]
#[rustc_nounwind]
#[rustc_do_not_const_check]
#[inline]
#[miri::intrinsic_fallback_is_spec]
pub const fn ptr_guaranteed_cmp<T>(ptr: *const T, other: *const T) -> u8 {
    (ptr == other) as u8
}

const CHECK:u8=ptr_guaranteed_cmp(0usize as *const u8,0usize as *const u8);
#[unsafe(no_mangle)]pub extern "C" fn observed(a:*const u8,b:*const u8)->u8{ptr_guaranteed_cmp(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn observed_const()->u8{CHECK}
