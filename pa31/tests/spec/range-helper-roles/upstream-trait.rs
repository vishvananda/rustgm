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
    fn sub_one(self) -> Self;
}
