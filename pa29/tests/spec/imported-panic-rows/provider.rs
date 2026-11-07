#![feature(no_core, lang_items, rustc_attrs, staged_api, allow_internal_unstable, decl_macro)]
#![no_core]
#![stable(feature = "panics", since = "1.0.0")]
#[lang = "pointee_sized"]
#[stable(feature = "panics", since = "1.0.0")]
pub trait PointeeSized {}
#[lang = "meta_sized"]
#[stable(feature = "panics", since = "1.0.0")]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
#[stable(feature = "panics", since = "1.0.0")]
pub trait Sized: MetaSized {}

#[rustc_builtin_macro]
#[macro_export]
#[stable(feature = "panics", since = "1.0.0")]
macro_rules! cfg { ($($cfg:tt)*) => { /* compiler built-in */ } }
// Core's shapes: each edition-panic contract is the `edition_panic` member of
// the row's allowance list.
#[rustc_builtin_macro(core_panic)]
#[macro_export]
#[allow_internal_unstable(edition_panic)]
#[stable(feature = "panics", since = "1.0.0")]
macro_rules! panic { ($($arg:tt)*) => { /* compiler built-in */ }; }
#[rustc_builtin_macro]
#[macro_export]
#[allow_internal_unstable(panic_internals, edition_panic)]
#[stable(feature = "panics", since = "1.0.0")]
macro_rules! assert {
    ($cond:expr $(,)?) => {{ /* compiler built-in */ }};
    ($cond:expr, $($arg:tt)+) => {{ /* compiler built-in */ }};
}
#[rustc_builtin_macro(unreachable)]
#[macro_export]
#[allow_internal_unstable(edition_panic)]
#[stable(feature = "panics", since = "1.0.0")]
macro_rules! unreachable { ($($arg:tt)*) => { /* compiler built-in */ }; }
#[macro_export]
#[allow_internal_unstable(edition_panic)]
#[stable(feature = "panics", since = "1.0.0")]
macro_rules! debug_assert {
    ($($arg:tt)*) => { if $crate::cfg!(debug_assertions) { $crate::assert!($($arg)*); } };
}
// The edition walk skips this wrapper and reads its caller's edition.
#[macro_export]
#[allow_internal_unstable(edition_panic)]
#[stable(feature = "panics", since = "1.0.0")]
macro_rules! fail_with { ($($arg:tt)*) => { $crate::panic!($($arg)*) }; }
// Without `edition_panic` the walk stops here, at this crate's edition.
#[macro_export]
#[stable(feature = "panics", since = "1.0.0")]
macro_rules! plain_fail { ($($arg:tt)*) => { $crate::panic!($($arg)*) }; }

#[stable(feature = "panics", since = "1.0.0")]
pub mod panicking {
    #[unstable(feature = "panic_internals", issue = "none")]
    pub fn formatted() -> u32 {
        42
    }
}

// Semiopaque `pub macro` rows; the values name the selected rule.
#[stable(feature = "panics", since = "1.0.0")]
pub mod panic {
    #[unstable(feature = "edition_panic", issue = "none")]
    #[allow_internal_unstable(panic_internals)]
    #[rustc_macro_transparency = "semiopaque"]
    pub macro panic_2015 {
        () => (45),
        ($($t:tt)+) => (46),
    }
    #[unstable(feature = "edition_panic", issue = "none")]
    #[allow_internal_unstable(panic_internals)]
    #[rustc_macro_transparency = "semiopaque"]
    pub macro panic_2021 {
        () => (41),
        ($($t:tt)+) => ($crate::panicking::formatted()),
    }
    #[unstable(feature = "edition_panic", issue = "none")]
    #[allow_internal_unstable(panic_internals)]
    #[rustc_macro_transparency = "semiopaque"]
    pub macro unreachable_2015 {
        () => (47),
        ($($t:tt)+) => (48),
    }
    #[unstable(feature = "edition_panic", issue = "none")]
    #[allow_internal_unstable(panic_internals)]
    #[rustc_macro_transparency = "semiopaque"]
    pub macro unreachable_2021 {
        () => (43),
        ($($t:tt)+) => (44),
    }
}
