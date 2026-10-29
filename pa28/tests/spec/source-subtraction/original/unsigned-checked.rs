        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_stable(feature = "const_checked_int_methods", since = "1.47.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
            // Per PR#103299, there's no advantage to the `overflowing` intrinsic
            // for *unsigned* subtraction and we just emit the manual check anyway.
            // Thus, rather than using `overflowing_sub` that produces a wrapping
            // subtraction, check it ourself so we can use an unchecked one.

            if self < rhs {
                None
            } else {
                // SAFETY: just checked this can't overflow
                Some(unsafe { intrinsics::unchecked_sub(self, rhs) })
            }
        }
