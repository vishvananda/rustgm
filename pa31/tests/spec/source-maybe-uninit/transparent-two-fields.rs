#![feature(no_core,lang_items,intrinsics,rustc_attrs,const_trait_impl,auto_traits,transparent_unions,decl_macro,allow_internal_unstable)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="unsize"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub trait Unsize<T:PointeeSized>:PointeeSized{}
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized{}
impl<'a,'b:'a,T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<&'a mut U> for &'b mut T{}
impl<T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<*mut U> for *mut T{}
#[lang="sync"]pub unsafe auto trait Sync{}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
pub mod intrinsics{use crate::Sized;#[rustc_intrinsic]pub const fn needs_drop<T:?Sized>()->bool;#[rustc_intrinsic]pub const fn size_of<T>()->usize;#[rustc_intrinsic]pub const fn align_of<T>()->usize;#[rustc_intrinsic]pub const unsafe fn transmute<A,B>(value:A)->B;#[rustc_intrinsic]pub const unsafe fn transmute_unchecked<A,B>(value:A)->B;#[rustc_intrinsic]pub const fn assert_inhabited<T>();}
pub const fn needs_drop<T: ?Sized>() -> bool {
    const { intrinsics::needs_drop::<T>() }
}
#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
#[lang = "drop_glue"]
pub(crate) const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{
    // Code here does not matter - this is replaced by the
    // real drop glue by the compiler.
}

impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}

impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
#[lang="not"]pub const trait Not{type Output;fn not(self)->Self::Output;}
const impl Not for bool{type Output=bool;fn not(self)->bool{!self}}
#[lang="eq"]pub const trait PartialEq<Rhs:PointeeSized=Self>:PointeeSized{fn eq(&self,other:&Rhs)->bool;fn ne(&self,other:&Rhs)->bool{!self.eq(other)}}
const impl PartialEq for usize{fn eq(&self,other:&Self)->bool{*self==*other}fn ne(&self,other:&Self)->bool{*self!=*other}}
pub trait SizedTypeProperties:Sized{#[lang="mem_size_const"]const SIZE:usize=intrinsics::size_of::<Self>();}
impl<T:Sized> SizedTypeProperties for T{}
#[rustc_do_not_const_check]#[rustc_const_stable_indirect]
#[lang="panic_display"]pub const fn panic_display<T:?Sized>(_:&T)->!{loop{}}
macro_rules! assert{($condition:expr)=>{if $condition {} else {panic_display(&"size mismatch")}}}
pub const unsafe fn transmute_neo<Src,Dst>(src:Src)->Dst{const{assert!(Src::SIZE==Dst::SIZE)};unsafe{intrinsics::transmute_unchecked(src)}}
#[lang="maybe_dangling"]#[repr(transparent)]pub struct MaybeDangling<P:?Sized>(P);
impl<P> MaybeDangling<P>{pub const fn new(p:P)->Self{Self(p)}}
#[lang="manually_drop"]#[repr(transparent)]pub struct ManuallyDrop<T:?Sized>{value:MaybeDangling<T>}
impl<T> ManuallyDrop<T>{pub const fn new(value:T)->ManuallyDrop<T>{ManuallyDrop{value:MaybeDangling::new(value)}}}
#[lang="bikeshed_guaranteed_no_drop"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]pub trait GuaranteedNoDrop{}
#[rustc_builtin_macro]pub macro derive($item:item){}
#[rustc_builtin_macro]#[allow_internal_unstable(core_intrinsics,derive_clone_copy_internals)]pub macro Copy($item:item){}
pub mod marker{pub use crate::Copy;}
impl<T:Copy> Clone for MaybeDangling<T>{fn clone(&self)->Self{*self}}
impl<T:Copy> Copy for MaybeDangling<T>{}
impl<T:Copy> Clone for ManuallyDrop<T>{fn clone(&self)->Self{*self}}
impl<T:Copy> Copy for ManuallyDrop<T>{}
#[repr(transparent)]union U{first:u8,second:u8}
