#![feature(no_core, lang_items, rustc_attrs, staged_api, allow_internal_unstable)]
#![no_core]
#![stable(feature = "apis", since = "1.0.0")]
#[lang = "pointee_sized"]
#[stable(feature = "apis", since = "1.0.0")]
pub trait PointeeSized {}
#[lang = "meta_sized"]
#[stable(feature = "apis", since = "1.0.0")]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
#[stable(feature = "apis", since = "1.0.0")]
pub trait Sized: MetaSized {}

#[rustc_builtin_macro]
#[macro_export]
#[stable(feature = "apis", since = "1.0.0")]
macro_rules! cfg { ($($cfg:tt)*) => { /* compiler built-in */ } }
#[rustc_builtin_macro]
#[macro_export]
#[stable(feature = "apis", since = "1.0.0")]
macro_rules! line { () => { /* compiler built-in */ } }

#[unstable(feature = "gated_api", issue = "none")]
pub fn seven() -> u32 {
    7
}
// The list admits the unstable function inside this row's expansion.
#[macro_export]
#[stable(feature = "apis", since = "1.0.0")]
#[allow_internal_unstable(gated_api)]
macro_rules! api { () => { $crate::seven() } }
#[macro_export]
#[stable(feature = "apis", since = "1.0.0")]
macro_rules! leaky_api { () => { $crate::seven() } }
