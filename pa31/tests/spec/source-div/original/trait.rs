#[lang = "div"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "cannot divide `{Self}` by `{Rhs}`",
    label = "no implementation for `{Self} / {Rhs}`"
)]
#[doc(alias = "/")]
pub const trait Div<Rhs = Self> {
    /// The resulting type after applying the `/` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the `/` operation.
    ///
    /// # Example
    ///
    /// ```
    /// assert_eq!(12 / 2, 6);
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[rustc_diagnostic_item = "div"]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn div(self, rhs: Rhs) -> Self::Output;
}


