#![feature(staged_api, auto_traits)]
#![unstable(feature="role_fixture", issue="none")]
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
#[lang="freeze"] pub unsafe auto trait Freeze {}

#[allow(non_camel_case_types)]
pub type c_char = i8;

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

