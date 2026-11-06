#[lang = "RangeCopy"]
pub struct Range<Idx> {
    /// The lower bound of the range (inclusive).
    #[stable(feature = "new_range_api", since = "1.96.0")]
    pub start: Idx,
    /// The upper bound of the range (exclusive).
    #[stable(feature = "new_range_api", since = "1.96.0")]
    pub end: Idx,
}
