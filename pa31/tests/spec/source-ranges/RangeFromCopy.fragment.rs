#[lang = "RangeFromCopy"]
pub struct RangeFrom<Idx> {
    /// The lower bound of the range (inclusive).
    #[stable(feature = "new_range_from_api", since = "1.96.0")]
    pub start: Idx,
}
