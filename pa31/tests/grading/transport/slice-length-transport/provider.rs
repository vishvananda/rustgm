#![feature(intrinsics, core_intrinsics, auto_traits, staged_api, const_destruct, unsize, coerce_unsized)]
#![rustc_coherence_is_core]
#![unstable(feature="pointer_metadata_fixture",issue="none")]
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
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[unstable(feature="const_destruct",issue="133214")]
#[rustc_const_unstable(feature="const_destruct",issue="133214")]
#[lang="destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct:PointeeSized{}
#[unstable(feature="unsize",issue="18598")]
#[lang="unsize"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub trait Unsize<T:PointeeSized>:PointeeSized{}
#[unstable(feature="coerce_unsized",issue="18598")]
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized{}
#[unstable(feature="coerce_unsized",issue="18598")]
impl<T:PointeeSized+Unsize<U>,U:PointeeSized>CoerceUnsized<*const U> for *const T{}
impl<T:PointeeSized>Clone for *const T{fn clone(&self)->Self{*self}}
impl<T:PointeeSized>Copy for *const T{}
impl Clone for usize{fn clone(&self)->Self{*self}}
impl Copy for usize{}
impl Clone for u8{fn clone(&self)->Self{*self}}
impl Copy for u8{}
#[lang="pointee_trait"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub trait Pointee:PointeeSized{
 #[lang="metadata_type"]type Metadata:Sized;
}
#[unstable(feature="core_intrinsics",issue="none")]
#[rustc_nounwind]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
pub const fn ptr_metadata<P:Pointee<Metadata=M>+PointeeSized,M>(ptr:*const P)->M;
// core::ptr::metadata's actual source wrapper body; generic result remains a projection.
pub const fn metadata<T:PointeeSized>(ptr:*const T)-><T as Pointee>::Metadata{ptr_metadata(ptr)}

mod ptr {
    pub use crate::metadata;
}

impl<T> [T] {
    #[lang = "slice_len_fn"]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_slice_len", since = "1.39.0")]
    #[rustc_no_implicit_autorefs]
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        ptr::metadata(self)
    }
}

#[unstable(feature = "coerce_unsized", issue = "18598")]
impl<'a, 'b: 'a, T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<&'a U> for &'b T {}
impl<T:PointeeSized>Clone for &T{fn clone(&self)->Self{*self}}
impl<T:PointeeSized>Copy for &T{}
#[lang="drop_glue"]pub(crate) const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
