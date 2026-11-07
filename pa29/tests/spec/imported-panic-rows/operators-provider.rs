#![feature(no_core, lang_items, rustc_attrs, staged_api, allow_internal_unstable)]
#![no_core]
#![stable(feature = "operators", since = "1.0.0")]
#[lang = "pointee_sized"]
#[stable(feature = "operators", since = "1.0.0")]
pub trait PointeeSized {}
#[lang = "meta_sized"]
#[stable(feature = "operators", since = "1.0.0")]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
#[stable(feature = "operators", since = "1.0.0")]
pub trait Sized: MetaSized {}
#[lang = "legacy_receiver"]
#[stable(feature = "operators", since = "1.0.0")]
pub trait LegacyReceiver {}
#[stable(feature = "operators", since = "1.0.0")]
impl<T> LegacyReceiver for &T {}
#[lang = "clone"]
#[stable(feature = "operators", since = "1.0.0")]
pub trait Clone: Sized {
    #[lang = "clone_fn"]
    #[stable(feature = "operators", since = "1.0.0")]
    fn clone(&self) -> Self;
}
#[lang = "copy"]
#[stable(feature = "operators", since = "1.0.0")]
pub trait Copy: Clone {}
#[stable(feature = "operators", since = "1.0.0")]
impl Clone for bool { fn clone(&self) -> Self { *self } }
#[stable(feature = "operators", since = "1.0.0")]
impl Copy for bool {}
#[lang = "not"]
#[stable(feature = "operators", since = "1.0.0")]
pub trait Not {
    #[stable(feature = "operators", since = "1.0.0")]
    type Output;
    #[stable(feature = "operators", since = "1.0.0")]
    fn not(self) -> Self::Output;
}
#[stable(feature = "operators", since = "1.0.0")]
impl Not for bool {
    type Output = bool;
    fn not(self) -> bool { !self }
}
// The operator items an executable `assert!` needs in a fresh consumer.
#[rustc_builtin_macro]
#[macro_export]
#[allow_internal_unstable(panic_internals, edition_panic)]
#[stable(feature = "operators", since = "1.0.0")]
macro_rules! assert {
    ($cond:expr $(,)?) => {{ /* compiler built-in */ }};
    ($cond:expr, $($arg:tt)+) => {{ /* compiler built-in */ }};
}
