#[doc(alias = "==")]
#[doc(alias = "!=")]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_diagnostic_item = "Eq"]
#[rustc_const_unstable(feature = "const_cmp", issue = "143800")]
pub const trait Eq: [const] PartialEq<Self> + PointeeSized {
    // This method was used solely by `#[derive(Eq)]` to assert that every component of a
    // type implements `Eq` itself.
    //
    // This should never be implemented by hand.
    #[doc(hidden)]
    #[coverage(off)]
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_diagnostic_item = "assert_receiver_is_total_eq"]
    #[deprecated(since = "1.95.0", note = "implementation detail of `#[derive(Eq)]`")]
    fn assert_receiver_is_total_eq(&self) {}

    // FIXME (#152504): this method is used solely by `#[derive(Eq)]` to assert that
    // every component of a type implements `Eq` itself. It will be removed again soon.
    #[doc(hidden)]
    #[coverage(off)]
    #[unstable(feature = "derive_eq_internals", issue = "none")]
    fn assert_fields_are_eq(&self) {}
}

