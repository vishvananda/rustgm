#![feature(no_core,lang_items,rustc_attrs,const_trait_impl,staged_api)]
#![no_core]#![allow(internal_features,dead_code)]
#![unstable(feature="range_role_fixture",issue="none")]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[unstable(feature = "pattern_type_range_trait", issue = "123646")]
#[rustc_const_unstable(feature = "pattern_type_range_trait", issue = "123646")]
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid base type for range patterns",
    label = "only integer types and `char` are supported"
)]
pub const trait RangePattern {
    /// Trait version of the inherent `MIN` assoc const.
    #[lang = "RangeMin"]
    const MIN: Self;

    /// Trait version of the inherent `MIN` assoc const.
    #[lang = "RangeMax"]
    const MAX: Self;

    /// A compile-time helper to subtract 1 for exclusive ranges.
    #[lang = "RangeSub"]
    #[track_caller]
    fn sub_one<T>(self) -> Self;
}
