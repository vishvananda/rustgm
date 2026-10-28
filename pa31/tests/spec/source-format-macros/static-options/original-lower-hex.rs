#[stable(feature = "rust1", since = "1.0.0")]
pub trait LowerHex: PointeeSized {
    #[doc = include_str!("fmt_trait_method_doc.md")]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result;
}
