#![rustc_coherence_is_core]
#![feature(no_core,lang_items,rustc_attrs,intrinsics,core_intrinsics,auto_traits,staged_api,const_trait_impl,const_destruct,unsize,coerce_unsized)]
#![no_core]
#![unstable(feature="pointer_metadata_fixture",issue="none")]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
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
#[lang="pointee_trait"]pub trait Pointee:PointeeSized{
 #[lang="metadata_type"]type Metadata:Sized;
}
#[unstable(feature="core_intrinsics",issue="none")]
#[rustc_nounwind]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
pub const fn ptr_metadata<P:Pointee<Metadata=M>+PointeeSized,M>(ptr:*const P)->M;
// core::ptr::metadata's actual source wrapper body; generic result remains a projection.
pub const fn metadata<T:PointeeSized>(ptr:*const T)-><T as Pointee>::Metadata{ptr_metadata(ptr)}
pub mod mem{#[stable(feature="rust1",since="1.0.0")]#[rustc_const_stable(feature="const_transmute",since="1.56.0")]#[rustc_intrinsic]pub const unsafe fn transmute<S,D>(src:S)->D;}
pub mod ptr{pub use crate::metadata;}
pub mod str{use crate::mem;impl str{
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_str_len", since = "1.39.0")]
    #[rustc_diagnostic_item = "str_len"]
    #[rustc_no_implicit_autorefs]
    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.as_bytes().len()
    }
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "str_as_bytes", since = "1.39.0")]
    #[must_use]
    #[inline(always)]
    #[allow(unused_attributes)]
    pub const fn as_bytes(&self) -> &[u8] {
        // SAFETY: const sound because we transmute two types with the same layout
        unsafe { mem::transmute(self) }
    }
}}
pub mod slice{use crate::ptr;impl<T>[T]{
    #[lang = "slice_len_fn"]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_slice_len", since = "1.39.0")]
    #[rustc_no_implicit_autorefs]
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        ptr::metadata(self)
    }
}}
pub const CONST_LEN:usize="💡".len();pub fn actual_runtime(s:&str)->usize{s.len()}#[unsafe(no_mangle)]pub extern "C" fn str_length()->usize{actual_runtime("héllo")}#[unsafe(no_mangle)]pub extern "C" fn slice_length()->usize{[1u8,2,3].len()}#[unsafe(no_mangle)]pub extern "C" fn const_length()->usize{CONST_LEN}

#[unstable(feature = "coerce_unsized", issue = "18598")]
impl<'a, 'b: 'a, T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<&'a U> for &'b T {}

impl<T:PointeeSized>Clone for &T{fn clone(&self)->Self{*self}}impl<T:PointeeSized>Copy for &T{}
impl<T:Copy,const N:usize>Clone for [T;N]{fn clone(&self)->Self{*self}}impl<T:Copy,const N:usize>Copy for [T;N]{}
#[lang="drop_glue"]pub(crate) const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
