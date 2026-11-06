#[lang = "RangeToInclusiveCopy"]
pub struct RangeToInclusive<Idx> {
    /// The upper bound of the range (inclusive)
    #[stable(feature = "new_range_to_inclusive_api", since = "1.96.0")]
    pub last: Idx,
}
