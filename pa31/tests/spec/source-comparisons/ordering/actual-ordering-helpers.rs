impl Ordering {
    #[inline]
    const fn as_raw(self) -> i8 {
        // FIXME(const-hack): just use `PartialOrd` against `Equal` once that's const
        crate::intrinsics::discriminant_value(&self)
    }

    /// Returns `true` if the ordering is the `Equal` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(Ordering::Less.is_eq(), false);
    /// assert_eq!(Ordering::Equal.is_eq(), true);
    /// assert_eq!(Ordering::Greater.is_eq(), false);
    /// ```
    #[inline]
    #[must_use]
    #[rustc_const_stable(feature = "ordering_helpers", since = "1.53.0")]
    #[stable(feature = "ordering_helpers", since = "1.53.0")]
    pub const fn is_eq(self) -> bool {
        // All the `is_*` methods are implemented as comparisons against zero
        // to follow how clang's libcxx implements their equivalents in
        // <https://github.com/llvm/llvm-project/blob/60486292b79885b7800b082754153202bef5b1f0/libcxx/include/__compare/is_eq.h#L23-L28>

        self.as_raw() == 0
    }

    /// Returns `true` if the ordering is not the `Equal` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(Ordering::Less.is_ne(), true);
    /// assert_eq!(Ordering::Equal.is_ne(), false);
    /// assert_eq!(Ordering::Greater.is_ne(), true);
    /// ```
    #[inline]
    #[must_use]
    #[rustc_const_stable(feature = "ordering_helpers", since = "1.53.0")]
    #[stable(feature = "ordering_helpers", since = "1.53.0")]
    pub const fn is_ne(self) -> bool {
        self.as_raw() != 0
    }

    /// Returns `true` if the ordering is the `Less` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(Ordering::Less.is_lt(), true);
    /// assert_eq!(Ordering::Equal.is_lt(), false);
    /// assert_eq!(Ordering::Greater.is_lt(), false);
    /// ```
    #[inline]
    #[must_use]
    #[rustc_const_stable(feature = "ordering_helpers", since = "1.53.0")]
    #[stable(feature = "ordering_helpers", since = "1.53.0")]
    pub const fn is_lt(self) -> bool {
        self.as_raw() < 0
    }

    /// Returns `true` if the ordering is the `Greater` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(Ordering::Less.is_gt(), false);
    /// assert_eq!(Ordering::Equal.is_gt(), false);
    /// assert_eq!(Ordering::Greater.is_gt(), true);
    /// ```
    #[inline]
    #[must_use]
    #[rustc_const_stable(feature = "ordering_helpers", since = "1.53.0")]
    #[stable(feature = "ordering_helpers", since = "1.53.0")]
    pub const fn is_gt(self) -> bool {
        self.as_raw() > 0
    }

    /// Returns `true` if the ordering is either the `Less` or `Equal` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(Ordering::Less.is_le(), true);
    /// assert_eq!(Ordering::Equal.is_le(), true);
    /// assert_eq!(Ordering::Greater.is_le(), false);
    /// ```
    #[inline]
    #[must_use]
    #[rustc_const_stable(feature = "ordering_helpers", since = "1.53.0")]
    #[stable(feature = "ordering_helpers", since = "1.53.0")]
    pub const fn is_le(self) -> bool {
        self.as_raw() <= 0
    }

    /// Returns `true` if the ordering is either the `Greater` or `Equal` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(Ordering::Less.is_ge(), false);
    /// assert_eq!(Ordering::Equal.is_ge(), true);
    /// assert_eq!(Ordering::Greater.is_ge(), true);
    /// ```
    #[inline]
    #[must_use]
    #[rustc_const_stable(feature = "ordering_helpers", since = "1.53.0")]
    #[stable(feature = "ordering_helpers", since = "1.53.0")]
    pub const fn is_ge(self) -> bool {
        self.as_raw() >= 0
    }
