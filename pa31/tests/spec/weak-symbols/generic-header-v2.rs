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
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_builtin_macro]
    #[macro_export]
    #[rustc_diagnostic_item = "include_str_macro"]
    macro_rules! include_str {
        ($file:expr $(,)?) => {{ /* compiler built-in */ }};
    }
pub mod ffi {
#[allow(non_camel_case_types)]pub type c_char=i8;
#[allow(non_camel_case_types)]pub type c_int=i32;
#[doc = include_str!("c_void.md")]
#[lang = "c_void"]
#[repr(u8)]
#[stable(feature = "core_c_void", since = "1.30.0")]
pub enum c_void {
    #[unstable(
        feature = "c_void_variant",
        reason = "temporary implementation detail",
        issue = "none"
    )]
    #[doc(hidden)]
    __variant1,
    #[unstable(
        feature = "c_void_variant",
        reason = "temporary implementation detail",
        issue = "none"
    )]
    #[doc(hidden)]
    __variant2,
}
}
mod runtime_symbols {
    use crate::ffi::{c_char, c_int, c_void};

    unsafe extern "C" {
        #[lang = "memcpy_fn"]
        fn memcpy<T>(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;

        #[lang = "memmove_fn"]
        fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;

        #[lang = "memset_fn"]
        fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;

        #[lang = "memcmp_fn"]
        fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> c_int;

        #[lang = "bcmp_fn"]
        fn bcmp(s1: *const c_void, s2: *const c_void, n: usize) -> c_int;

        #[lang = "strlen_fn"]
        fn strlen(s: *const c_char) -> usize;
    }
}
