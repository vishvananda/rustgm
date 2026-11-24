#![feature(no_core, lang_items, rustc_attrs, staged_api, auto_traits, pattern_types)]
#![no_core]
#![stable(feature = "base", since = "1.0.0")]
#![allow(internal_features, dead_code)]
#![deny(unfulfilled_lint_expectations)]

#[lang = "pointee_sized"]
#[stable(feature = "base", since = "1.0.0")]
pub trait PointeeSized {}
#[lang = "meta_sized"]
#[stable(feature = "base", since = "1.0.0")]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
#[stable(feature = "base", since = "1.0.0")]
pub trait Sized: MetaSized {}
#[lang = "legacy_receiver"]
#[stable(feature = "base", since = "1.0.0")]
pub trait LegacyReceiver: PointeeSized {}
#[stable(feature = "base", since = "1.0.0")]
impl<T: PointeeSized> LegacyReceiver for &T {}
#[lang = "clone"]
#[stable(feature = "base", since = "1.0.0")]
pub trait Clone: Sized {
    #[lang = "clone_fn"]
    #[stable(feature = "base", since = "1.0.0")]
    fn clone(&self) -> Self;
}
#[lang = "copy"]
#[stable(feature = "base", since = "1.0.0")]
pub trait Copy: Clone {}
#[lang = "freeze"]
#[stable(feature = "base", since = "1.0.0")]
pub unsafe auto trait Freeze {}

#[macro_export]
#[rustc_builtin_macro(pattern_type)]
#[stable(feature = "base", since = "1.0.0")]
macro_rules! pattern_type {
    ($($tokens:tt)*) => {};
}

#[stable(feature = "base", since = "1.0.0")]
trait Checked {}
#[cfg_attr(unstable_pointee, unstable(feature = "sample", issue = "none"))]
#[cfg_attr(not(unstable_pointee), stable(feature = "base", since = "1.0.0"))]
struct Pointee;

#[unstable(feature = "sample", issue = "none")]
impl Checked for pattern_type!(*const Pointee is !null) {}
