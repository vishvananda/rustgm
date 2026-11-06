#![feature(no_core,lang_items,rustc_attrs,decl_macro,derive_coerce_pointee,coerce_pointee_validated,unsize,coerce_unsized,dispatch_from_dyn,staged_api,allow_internal_unstable)]
#![no_core]
#![unstable(feature="derive_fixture",issue="none")]
#![allow(internal_features,dead_code)]
extern crate self as core;
pub mod marker{
 #[lang="pointee_sized"]pub trait PointeeSized{}
 #[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
 #[lang="sized"]pub trait Sized:MetaSized{}
 #[unstable(feature="unsize",issue="18598")]#[lang="unsize"]#[rustc_deny_explicit_impl]pub trait Unsize<T:PointeeSized>:PointeeSized{}
#[lang = "coerce_pointee_validated"]
#[unstable(feature = "coerce_pointee_validated", issue = "none")]
#[doc(hidden)]
pub trait CoercePointeeValidated {
    /* compiler built-in */
}

 #[rustc_builtin_macro(CoercePointee,attributes(pointee))]
 #[allow_internal_unstable(dispatch_from_dyn,coerce_unsized,unsize,coerce_pointee_validated)]
 #[unstable(feature="derive_coerce_pointee",issue="123430")]pub macro CoercePointee($item:item){}
}
pub mod ops{
 #[unstable(feature="coerce_unsized",issue="18598")]#[lang="coerce_unsized"]pub trait CoerceUnsized<T:crate::marker::PointeeSized>:crate::marker::Sized{}
 #[unstable(feature="dispatch_from_dyn",issue="18598")]#[lang="dispatch_from_dyn"]pub trait DispatchFromDyn<T:crate::marker::PointeeSized>{}
 impl<T:?crate::marker::Sized+crate::marker::Unsize<U>,U:?crate::marker::Sized> CoerceUnsized<*const U> for *const T{}
 impl<T:?crate::marker::Sized+crate::marker::Unsize<U>,U:?crate::marker::Sized> DispatchFromDyn<*const U> for *const T{}
}
#[rustc_builtin_macro]pub macro derive($item:item){}
use marker::Sized;
#[derive(marker::CoercePointee)]
#[repr(transparent)]
pub struct Ptr<T:?Sized>(*const T);
