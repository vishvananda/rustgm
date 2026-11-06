#[lang = "RangeInclusiveCopy"]
pub struct RangeInclusive<Idx> {
    /// The lower bound of the range (inclusive).
    #[stable(feature = "new_range_inclusive_api", since = "1.95.0")]
    pub start: Idx,
    /// The upper bound of the range (inclusive).
    #[stable(feature = "new_range_inclusive_api", since = "1.95.0")]
    pub last: Idx,
}
