#[lang = "range_inclusive_new"]
    #[stable(feature = "inclusive_range_methods", since = "1.27.0")]
    #[inline]
    #[rustc_promotable]
    #[rustc_const_stable(feature = "const_range_new", since = "1.32.0")]
    pub const fn new(start: Idx, end: Idx) -> Self {
        Self { start, end, exhausted: false }
    }
