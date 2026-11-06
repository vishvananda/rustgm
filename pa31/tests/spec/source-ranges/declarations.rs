#![feature(const_destruct)]
#![feature(staged_api,const_trait_impl,auto_traits,negative_impls)]
#![unstable(feature="marker_fixture",issue="none")]
#![feature(no_core,lang_items,rustc_attrs,fundamental)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
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
pub trait Clone: Sized { #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"]
pub trait Copy: Clone {}

#[lang="freeze"]pub unsafe auto trait Freeze{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[unstable(feature = "const_destruct", issue = "133214")]
#[rustc_const_unstable(feature = "const_destruct", issue = "133214")]
#[lang = "destruct"]
#[diagnostic::on_unimplemented(message = "can't drop `{Self}`")]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}
pub mod old{#[lang = "RangeFrom"]
pub struct RangeFrom<Idx> {
    /// The lower bound of the range (inclusive).
    #[stable(feature = "rust1", since = "1.0.0")]
    pub start: Idx,
}#[lang = "RangeFull"]
pub struct RangeFull;#[lang = "RangeInclusive"]
pub struct RangeInclusive<Idx> {
    // Note that the fields here are not public to allow changing the
    // representation in the future; in particular, while we could plausibly
    // expose start/end, modifying them without changing (future/current)
    // private fields may lead to incorrect behavior, so we don't want to
    // support that mode.
    pub(crate) start: Idx,
    pub(crate) end: Idx,

    // This field is:
    //  - `false` upon construction
    //  - `false` when iteration has yielded an element and the iterator is not exhausted
    //  - `true` when iteration has been used to exhaust the iterator
    //
    // This is required to support PartialEq and Hash without a PartialOrd bound or specialization.
    pub(crate) exhausted: bool,
}#[lang = "Range"]
pub struct Range<Idx> {
    /// The lower bound of the range (inclusive).
    #[stable(feature = "rust1", since = "1.0.0")]
    pub start: Idx,
    /// The upper bound of the range (exclusive).
    #[stable(feature = "rust1", since = "1.0.0")]
    pub end: Idx,
}#[lang = "RangeToInclusive"]
pub struct RangeToInclusive<Idx> {
    /// The upper bound of the range (inclusive)
    #[stable(feature = "inclusive_range", since = "1.26.0")]
    pub end: Idx,
}#[lang = "RangeTo"]
pub struct RangeTo<Idx> {
    /// The upper bound of the range (exclusive).
    #[stable(feature = "rust1", since = "1.0.0")]
    pub end: Idx,
}impl<Idx> RangeInclusive<Idx>{#[lang = "range_inclusive_new"]
    #[stable(feature = "inclusive_range_methods", since = "1.27.0")]
    #[inline]
    #[rustc_promotable]
    #[rustc_const_stable(feature = "const_range_new", since = "1.32.0")]
    pub const fn new(start: Idx, end: Idx) -> Self {
        Self { start, end, exhausted: false }
    }}}
pub mod new{#[lang = "RangeFromCopy"]
pub struct RangeFrom<Idx> {
    /// The lower bound of the range (inclusive).
    #[stable(feature = "new_range_from_api", since = "1.96.0")]
    pub start: Idx,
}#[lang = "RangeCopy"]
pub struct Range<Idx> {
    /// The lower bound of the range (inclusive).
    #[stable(feature = "new_range_api", since = "1.96.0")]
    pub start: Idx,
    /// The upper bound of the range (exclusive).
    #[stable(feature = "new_range_api", since = "1.96.0")]
    pub end: Idx,
}#[lang = "RangeInclusiveCopy"]
pub struct RangeInclusive<Idx> {
    /// The lower bound of the range (inclusive).
    #[stable(feature = "new_range_inclusive_api", since = "1.95.0")]
    pub start: Idx,
    /// The upper bound of the range (inclusive).
    #[stable(feature = "new_range_inclusive_api", since = "1.95.0")]
    pub last: Idx,
}#[lang = "RangeToInclusiveCopy"]
pub struct RangeToInclusive<Idx> {
    /// The upper bound of the range (inclusive)
    #[stable(feature = "new_range_to_inclusive_api", since = "1.96.0")]
    pub last: Idx,
}}
