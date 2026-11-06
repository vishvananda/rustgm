#![feature(no_core,lang_items,rustc_attrs,staged_api,auto_traits,const_trait_impl)]
#![no_core]
#![rustc_coherence_is_core]
#![unstable(feature="cstr_fixture",issue="none")]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="sync"]pub unsafe auto trait Sync{}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
/// Helper function for `drop_in_place`. The compiler replaces this by the actual drop glue.
#[lang = "drop_glue"]
pub(crate) const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{
    // Code here does not matter - this is replaced by the
    // real drop glue by the compiler.
}

#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for i8{fn clone(&self)->Self{*self}}impl Copy for i8{}
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
impl<T:PointeeSized>Clone for &T{fn clone(&self)->Self{*self}}impl<T:PointeeSized>Copy for &T{}
impl<T:PointeeSized>Clone for *const T{fn clone(&self)->Self{*self}}impl<T:PointeeSized>Copy for *const T{}
pub mod ffi {
#[allow(non_camel_case_types)]pub type c_char=i8;
#[stable(feature = "core_c_str", since = "1.64.0")]
#[rustc_diagnostic_item = "cstr_type"]
#[rustc_has_incoherent_inherent_impls]
#[lang = "CStr"]
// `fn from` in `impl From<&CStr> for Box<CStr>` current implementation relies
// on `CStr` being layout-compatible with `[u8]`.
// However, `CStr` layout is considered an implementation detail and must not be relied upon. We
// want `repr(transparent)` but we don't want it to show up in rustdoc, so we hide it under
// `cfg(doc)`. This is an ad-hoc implementation of attribute privacy.
#[repr(transparent)]
pub struct CStr {
    // FIXME: this should not be represented with a DST slice but rather with
    //        just a raw `c_char` along with some form of marker to make
    //        this an unsized type. Essentially `sizeof(&CStr)` should be the
    //        same as `sizeof(&c_char)` but `CStr` should be an unsized type.
    inner: [c_char],
}
impl CStr {
    #[inline]
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_str_as_ptr", since = "1.32.0")]
    #[rustc_as_ptr]
    #[rustc_never_returns_null_ptr]
    pub const fn as_ptr(&self) -> *const c_char {
        self.inner.as_ptr()
    }
    #[inline]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_cstr_methods", since = "1.72.0")]
    pub const fn to_bytes_with_nul(&self) -> &[u8] {
        // SAFETY: Transmuting a slice of `c_char`s to a slice of `u8`s
        // is safe on all supported targets.
        unsafe { &*((&raw const self.inner) as *const [u8]) }
    }
}
}
impl<T> [T] {
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_slice_as_ptr", since = "1.32.0")]
    #[rustc_never_returns_null_ptr]
    #[rustc_as_ptr]
    #[inline(always)]
    #[must_use]
    pub const fn as_ptr(&self) -> *const T {
        self as *const [T] as *const T
    }
}
pub fn bad(x:&ffi::CStr)->&[i8]{&x.inner}
