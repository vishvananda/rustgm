        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_stable(feature = "const_checked_int_methods", since = "1.47.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
            let (a, b) = self.overflowing_sub(rhs);
            if intrinsics::unlikely(b) { None } else { Some(a) }
        }
