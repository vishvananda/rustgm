#[allow(unused)]
macro_rules! types {
    (
        #![$stability_first:meta]
        $(
            #![$stability_more:meta]
        )*

        $(
            $(#[$doc:meta])*
            $(stability: [$stability_already: meta])*
            pub struct $name:ident($len:literal x $v:vis $elem_type:ty);
        )*
    ) => (types! {
        $(
            #![$stability_more]
        )*

        $(
            $(#[$doc])*
            $(stability: [$stability_already])*
            stability: [$stability_first]
            pub struct $name($len x $v $elem_type);
        )*
    });

    (
        $(
            $(#[$doc:meta])*
            $(stability: [$stability: meta])+
            pub struct $name:ident($len:literal x $v:vis $elem_type:ty);
        )*
    ) => ($(
        $(#[$doc])*
        $(#[$stability])+
        #[derive(Copy, Clone)]
        #[allow(non_camel_case_types)]
        #[repr(simd)]
        #[allow(clippy::missing_inline_in_public_items)]
        pub struct $name($v [$elem_type; $len]);

        impl $name {
            /// Put the same value in every lane.
            #[inline(always)]
            $v fn splat(value: $elem_type) -> $name {
                unsafe { $crate::intrinsics::simd::simd_splat(value) }
            }

            /// Returns an array reference containing the entire SIMD vector.
            $v const fn as_array(&self) -> &[$elem_type; $len] {
                // SAFETY: this type is just an overaligned `[T; N]` with
                // potential padding at the end, so pointer casting to a
                // `&[T; N]` is safe.
                //
                // NOTE: This deliberately doesn't just use `&self.0` because it may soon be banned
                // see https://github.com/rust-lang/compiler-team/issues/838
                unsafe { &*(self as *const Self as *const [$elem_type; $len]) }

            }

            /// Returns a mutable array reference containing the entire SIMD vector.
            #[inline]
            $v fn as_mut_array(&mut self) -> &mut [$elem_type; $len] {
                // SAFETY: this type is just an overaligned `[T; N]` with
                // potential padding at the end, so pointer casting to a
                // `&mut [T; N]` is safe.
                //
                // NOTE: This deliberately doesn't just use `&mut self.0` because it may soon be banned
                // see https://github.com/rust-lang/compiler-team/issues/838
                unsafe { &mut *(self as *mut Self as *mut [$elem_type; $len]) }
            }
        }

        $(#[$stability])+
        impl crate::fmt::Debug for $name {
            #[inline]
            fn fmt(&self, f: &mut crate::fmt::Formatter<'_>) -> crate::fmt::Result {
                crate::core_arch::simd::debug_simd_finish(f, stringify!($name), self.as_array())
            }
        }

        $(#[$stability])+
        impl crate::convert::From<crate::core_arch::simd::Simd<$elem_type, $len>> for $name {
            #[inline(always)]
            fn from(simd: crate::core_arch::simd::Simd<$elem_type, $len>) -> Self {
                unsafe { crate::mem::transmute(simd) }
            }
        }

        $(#[$stability])+
        impl crate::convert::From<$name> for crate::core_arch::simd::Simd<$elem_type, $len> {
            #[inline(always)]
            fn from(simd: $name) -> Self {
                unsafe { crate::mem::transmute(simd) }
            }
        }
    )*);
}
