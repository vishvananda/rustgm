macro_rules! add_impl {
    ($($t:ty)*) => ($(
        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")]
        const impl Add for $t {
            type Output = $t;

            #[inline]
            #[track_caller]
            #[rustc_inherit_overflow_checks]
            fn add(self, other: $t) -> $t { self + other }
        }

        forward_ref_binop! { impl Add, add for $t, $t,
        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")] }
    )*)
}
