macro_rules! define_valid_range_type {
    ($(
        $(#[$m:meta])*
        $vis:vis struct $name:ident($int:ident is $pat:pat);
    )+) => {$(
        #[derive(Clone, Copy)]
        #[repr(transparent)]
        $(#[$m])*
        $vis struct $name(pattern_type!($int is $pat));
        impl $name {
            #[inline]
            pub const fn new(val: $int) -> Option<Self> {
                #[allow(non_contiguous_range_endpoints)]
                if let $pat = val {
                    // SAFETY: just checked that the value matches the pattern
                    Some(unsafe { $name(crate::mem::transmute(val)) })
                } else {
                    None
                }
            }

            /// Constructs an instance of this type from the underlying integer
            /// primitive without checking whether its valid.
            ///
            /// # Safety
            /// Immediate language UB if `val` is not within the valid range for this
            /// type, as it violates the validity invariant.
            #[inline]
            pub const unsafe fn new_unchecked(val: $int) -> Self {
                // SAFETY: Caller promised that `val` is within the valid range.
                unsafe { crate::mem::transmute(val) }
            }

            #[inline]
            pub const fn as_inner(self) -> $int {
                // SAFETY: pattern types are always legal values of their base type
                // (Not using `.0` because that has perf regressions.)
                unsafe { crate::mem::transmute(self) }
            }
        }

        // This is required to allow matching a constant.  We don't get it from a derive
        // because the derived `PartialEq` would do a field projection, which is banned
        // by <https://github.com/rust-lang/compiler-team/issues/807>.
        impl StructuralPartialEq for $name {}

        impl Eq for $name {}

        impl PartialEq for $name {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.as_inner() == other.as_inner()
            }
        }

        impl Ord for $name {
            #[inline]
            fn cmp(&self, other: &Self) -> Ordering {
                Ord::cmp(&self.as_inner(), &other.as_inner())
            }
        }

        impl PartialOrd for $name {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(Ord::cmp(self, other))
            }
        }

        impl Hash for $name {
            // Required method
            fn hash<H: Hasher>(&self, state: &mut H) {
                Hash::hash(&self.as_inner(), state);
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                <$int as fmt::Debug>::fmt(&self.as_inner(), f)
            }
        }
    )+};
}

define_valid_range_type! {
    pub struct Nanoseconds(u32 is 0..=999_999_999);
}
