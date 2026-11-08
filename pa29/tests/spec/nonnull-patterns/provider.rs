#![feature(no_core,lang_items,rustc_attrs,fundamental,auto_traits)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#![feature(pattern_types)]
#[lang = "pointee_sized"]
#[diagnostic::on_unimplemented(
    message = "values of type `{Self}` may or may not have a size",
    label = "may or may not have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_coinductive]
pub trait PointeeSized {
    // Empty
}
#[lang = "meta_sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known",
    label = "doesn't have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
// `MetaSized` being coinductive, despite having supertraits, is okay for the same reasons as
// `Sized` above.
#[rustc_coinductive]
pub trait MetaSized: PointeeSized {
    // Empty
}
#[lang = "sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known at compilation time",
    label = "doesn't have a size known at compile-time"
)]
#[fundamental] // for Default, for example, which requires that `[T]: !Default` be evaluatable
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
// `Sized` being coinductive, despite having supertraits, is okay as there are no user-written impls,
// and we know that the supertraits are always implemented if the subtrait is just by looking at
// the builtin impls.
#[rustc_coinductive]
pub trait Sized: MetaSized {
    // Empty.
}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "clone"]
pub trait Clone: Sized {  #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"]
pub trait Copy: Clone {}


#[lang = "freeze"]
pub unsafe auto trait Freeze {}

// Authored source transport controls; sizing and receiver declarations above are original core contracts.
pub struct Packet<T> { pub value: T }
pub struct Hidden<T> { value: T }
pub fn make<T>(value: T) -> Packet<T> { Packet { value } }
pub fn take<T>(packet: Packet<T>) -> T { packet.value }
pub fn hide<T>(value: T) -> Hidden<T> { Hidden { value } }
// Authored pattern-type and NonNull transport controls.
// Core's `pattern_type` builtin macro declaration, without its stability
// attribute.
#[macro_export]
#[rustc_builtin_macro(pattern_type)]
macro_rules! pattern_type {
    ($($arg:tt)*) => {
        /* compiler built-in */
    };
}
// Core's `NonNull` declaration, without its stability attribute.
#[repr(transparent)]
#[rustc_nonnull_optimization_guaranteed]
#[rustc_diagnostic_item = "NonNull"]
pub struct NonNull<T: PointeeSized> {
    pointer: crate::pattern_type!(*const T is !null),
}
// Core's closed niche types, as `define_valid_range_type!` expands them,
// without their methods and impls.
#[repr(transparent)]
pub struct Nanoseconds(pattern_type!(u32 is 0..=999_999_999));
#[repr(transparent)]
pub struct NonZeroCharInner(pattern_type!(char is '\u{1}' ..= '\u{10ffff}'));
#[repr(transparent)]
pub struct CodePointInner(pattern_type!(u32 is 0..=0x10ffff));
// Authored closed unsigned and signed ranges.
#[repr(transparent)]
pub struct NonZeroU8(pattern_type!(u8 is 1..=255));
#[repr(transparent)]
pub struct Percent(pattern_type!(i8 is 0..=100));
