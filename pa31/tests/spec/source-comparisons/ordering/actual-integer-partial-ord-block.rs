            #[stable(feature = "rust1", since = "1.0.0")]
            #[rustc_const_unstable(feature = "const_cmp", issue = "143800")]
            const impl PartialOrd for $t {
                #[inline]
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    Some(crate::intrinsics::three_way_compare(*self, *other))
                }

                partial_ord_methods_primitive_impl!();
            }

