    #[must_use]
    #[inline]
    #[stable(feature = "is_some_and", since = "1.70.0")]
    #[rustc_const_unstable(feature = "const_option_ops", issue = "143956")]
    pub const fn is_some_and(self, f: impl [const] FnOnce(T) -> bool + [const] Destruct) -> bool {
        match self {
            None => false,
            Some(x) => f(x),
        }
    }
