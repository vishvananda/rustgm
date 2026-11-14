        #[track_caller]
        pub const unsafe fn unchecked_shl(self, rhs: u32) -> Self {
            assert_unsafe_precondition!(
                check_language_ub,
                concat!(stringify!($SelfT), "::unchecked_shl cannot overflow"),
                (
                    rhs: u32 = rhs,
                ) => rhs < <$ActualT>::BITS,
            );

            // SAFETY: this is guaranteed to be safe by the caller.
            unsafe {
                intrinsics::unchecked_shl(self, rhs)
            }
        }