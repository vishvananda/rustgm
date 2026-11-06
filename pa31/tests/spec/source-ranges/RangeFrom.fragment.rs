#[lang = "RangeFrom"]
pub struct RangeFrom<Idx> {
    /// The lower bound of the range (inclusive).
    #[stable(feature = "rust1", since = "1.0.0")]
    pub start: Idx,
}
