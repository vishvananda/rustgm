    #[stable(feature = "nonzero", since = "1.28.0")]
    #[rustc_const_stable(feature = "const_nonzero_int_methods", since = "1.47.0")]
    #[must_use]
    #[inline]
    pub const fn new(n: T) -> Option<Self> {
        // SAFETY: Memory layout optimization guarantees that `Option<NonZero<T>>` has
        //         the same layout and size as `T`, with `0` representing `None`.
        unsafe { intrinsics::transmute_unchecked(n) }
    }
