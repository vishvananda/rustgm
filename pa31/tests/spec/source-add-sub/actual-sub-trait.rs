#[lang = "sub"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "cannot subtract `{Rhs}` from `{Self}`",
    label = "no implementation for `{Self} - {Rhs}`"
)]
#[doc(alias = "-")]
pub const trait Sub<Rhs = Self> {
    /// The resulting type after applying the `-` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the `-` operation.
    ///
    /// # Example
    ///
    /// ```
    /// assert_eq!(12 - 1, 11);
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[rustc_diagnostic_item = "sub"]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn sub(self, rhs: Rhs) -> Self::Output;
}
