    #[stable(feature = "nonzero", since = "1.28.0")]
    #[rustc_const_stable(feature = "const_nonzero_get", since = "1.34.0")]
    #[inline]
    pub const fn get(self) -> T {
        // Rustc can set range metadata only if it loads `self` from
        // memory somewhere. If the value of `self` was from by-value argument
        // of some not-inlined function, LLVM don't have range metadata
        // to understand that the value cannot be zero.
        //
        // Using the transmute `assume`s the range at runtime.
        //
        // Even once LLVM supports `!range` metadata for function arguments
        // (see <https://github.com/llvm/llvm-project/issues/76628>), this can't
        // be `.0` because MCP#807 bans field-projecting into `scalar_valid_range`
        // types, and it arguably wouldn't want to be anyway because if this is
        // MIR-inlined, there's no opportunity to put that argument metadata anywhere.
        //
        // The good answer here will eventually be pattern types, which will hopefully
        // allow it to go back to `.0`, maybe with a cast of some sort.
        //
        // SAFETY: `ZeroablePrimitive` guarantees that the size and bit validity
        // of `.0` is such that this transmute is sound.
        unsafe { intrinsics::transmute_unchecked(self) }
    }
