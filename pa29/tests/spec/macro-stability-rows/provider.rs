#![feature(no_core, lang_items, rustc_attrs, staged_api, allow_internal_unstable)]
#![no_core]
#![stable(feature = "gates", since = "1.0.0")]
#[lang = "pointee_sized"]
#[stable(feature = "gates", since = "1.0.0")]
pub trait PointeeSized {}
#[lang = "meta_sized"]
#[stable(feature = "gates", since = "1.0.0")]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
#[stable(feature = "gates", since = "1.0.0")]
pub trait Sized: MetaSized {}

#[rustc_builtin_macro]
#[macro_export]
#[stable(feature = "gates", since = "1.0.0")]
macro_rules! cfg { ($($cfg:tt)*) => { /* compiler built-in */ } }
// An unstable builtin row: its consumer must enable `gated_line`.
#[rustc_builtin_macro]
#[macro_export]
#[unstable(feature = "gated_line", issue = "none")]
macro_rules! line { () => { /* compiler built-in */ } }
// A builtin row with an allowance list, as core's `assert` has.
#[rustc_builtin_macro]
#[macro_export]
#[stable(feature = "gates", since = "1.0.0")]
#[allow_internal_unstable(gated_answer)]
macro_rules! stringify { ($($t:tt)*) => { /* compiler built-in */ } }

#[macro_export]
#[unstable(feature = "gated_answer", issue = "none")]
macro_rules! answer { () => { 7 } }
#[macro_export]
#[stable(feature = "gates", since = "1.0.0")]
#[allow_internal_unstable(cfg_ub_checks)]
macro_rules! checked { () => { $crate::cfg!(ub_checks) } }
#[macro_export]
#[stable(feature = "gates", since = "1.0.0")]
#[allow_internal_unstable(gated_answer)]
macro_rules! wrapper { () => { $crate::answer!() } }
#[macro_export]
#[stable(feature = "gates", since = "1.0.0")]
#[allow_internal_unstable(gated_line)]
macro_rules! line_wrapper { () => { $crate::line!() } }
// No allowance list: its expansion's call of `answer` stays gated.
#[macro_export]
#[stable(feature = "gates", since = "1.0.0")]
macro_rules! leaky { () => { $crate::answer!() } }

#[stable(feature = "gates", since = "1.0.0")]
pub mod reexported {
    // A staged re-export carries its own stability; calls still check the
    // definition row's.
    #[stable(feature = "gates", since = "1.0.0")]
    pub use crate::answer;
}
