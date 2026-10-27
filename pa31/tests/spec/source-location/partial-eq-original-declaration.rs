#[lang = "eq"]
#[stable(feature = "rust1", since = "1.0.0")]
#[doc(alias = "==")]
#[doc(alias = "!=")]
#[diagnostic::on_unimplemented(
    message = "can't compare `{Self}` with `{Rhs}`",
    label = "no implementation for `{Self} == {Rhs}`"
)]
#[rustc_diagnostic_item = "PartialEq"]
#[rustc_const_unstable(feature = "const_cmp", issue = "143800")]
pub const trait PartialEq<Rhs: PointeeSized = Self>: PointeeSized {
    /// Tests for `self` and `other` values to be equal, and is used by `==`.
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_diagnostic_item = "cmp_partialeq_eq"]
    fn eq(&self, other: &Rhs) -> bool;

    /// Tests for `!=`. The default implementation is almost always sufficient,
    /// and should not be overridden without very good reason.
    #[inline]
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_diagnostic_item = "cmp_partialeq_ne"]
    fn ne(&self, other: &Rhs) -> bool {
        !self.eq(other)
    }
}
