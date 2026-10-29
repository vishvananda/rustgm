macro_rules! div_impl_integer {
    ($(($($t:ty)*) => $panic:expr),*) => ($($(
        /// This operation rounds towards zero, truncating any
        /// fractional part of the exact result.
        ///
        /// # Panics
        ///
        #[doc = $panic]
        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")]
        const impl Div for $t {
            type Output = $t;

            #[inline]
            #[track_caller]
            fn div(self, other: $t) -> $t { self / other }
        }

        forward_ref_binop! { impl Div, div for $t, $t,
        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")] }
    )*)*)
}

div_impl_integer! {
    (usize u8 u16 u32 u64 u128) => "This operation will panic if `other == 0`.",
    (isize i8 i16 i32 i64 i128) => "This operation will panic if `other == 0` or the division results in overflow."
}


