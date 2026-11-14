        /// Unchecked integer multiplication. Computes `self * rhs`, assuming overflow
        /// cannot occur.
        ///
        /// Calling `x.unchecked_mul(y)` is semantically equivalent to calling
        /// `x.`[`checked_mul`]`(y).`[`unwrap_unchecked`]`()`.
        ///
        /// If you're just trying to avoid the panic in debug mode, then **do not**
        /// use this.  Instead, you're looking for [`wrapping_mul`].
        ///
        /// # Safety
        ///
        /// This results in undefined behavior when
        #[doc = concat!("`self * rhs > ", stringify!($SelfT), "::MAX`,")]
        /// i.e. when [`checked_mul`] would return `None`.
        ///
        /// [`unwrap_unchecked`]: option/enum.Option.html#method.unwrap_unchecked
        #[doc = concat!("[`checked_mul`]: ", stringify!($SelfT), "::checked_mul")]
        #[doc = concat!("[`wrapping_mul`]: ", stringify!($SelfT), "::wrapping_mul")]
        #[stable(feature = "unchecked_math", since = "1.79.0")]
        #[rustc_const_stable(feature = "unchecked_math", since = "1.79.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        #[track_caller]
        pub const unsafe fn unchecked_mul(self, rhs: Self) -> Self {
            assert_unsafe_precondition!(
                check_language_ub,
                concat!(stringify!($SelfT), "::unchecked_mul cannot overflow"),
                (
                    lhs: $SelfT = self,
                    rhs: $SelfT = rhs,
                ) => !lhs.overflowing_mul(rhs).1,
            );

            // SAFETY: this is guaranteed to be safe by the caller.
            unsafe {
                intrinsics::unchecked_mul(self, rhs)
            }
        }

