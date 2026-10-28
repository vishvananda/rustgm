#[lang = "add"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[rustc_on_unimplemented(
    on(all(Self = "{integer}", Rhs = "{float}"), message = "cannot add a float to an integer",),
    on(all(Self = "{float}", Rhs = "{integer}"), message = "cannot add an integer to a float",),
    message = "cannot add `{Rhs}` to `{Self}`",
    label = "no implementation for `{Self} + {Rhs}`"
)]
#[doc(alias = "+")]
pub const trait Add<Rhs = Self> {
    /// The resulting type after applying the `+` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the `+` operation.
    ///
    /// # Example
    ///
    /// ```
    /// assert_eq!(12 + 1, 13);
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[rustc_diagnostic_item = "add"]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn add(self, rhs: Rhs) -> Self::Output;
}
