pub impl(self) unsafe trait ZeroablePrimitive: Sized + Copy {
    /// A type like `Self` but with a niche that includes zero.
    type NonZeroInner: Sized + Copy;
}

macro_rules! impl_zeroable_primitive {
    ($($NonZeroInner:ident ( $primitive:ty )),+ $(,)?) => {
        $(
            #[unstable(
                feature = "nonzero_internals",
                reason = "implementation detail which may disappear or be replaced at any time",
                issue = "none"
            )]
            unsafe impl ZeroablePrimitive for $primitive {
                type NonZeroInner = super::niche_types::$NonZeroInner;
            }
        )+
    };
}

impl_zeroable_primitive!(
    NonZeroU8Inner(u8),
    NonZeroU16Inner(u16),
    NonZeroU32Inner(u32),
    NonZeroU64Inner(u64),
    NonZeroU128Inner(u128),
    NonZeroUsizeInner(usize),
    NonZeroI8Inner(i8),
    NonZeroI16Inner(i16),
    NonZeroI32Inner(i32),
    NonZeroI64Inner(i64),
    NonZeroI128Inner(i128),
    NonZeroIsizeInner(isize),
    NonZeroCharInner(char),
);

/// A value that is known not to equal zero.
///
/// This enables some memory layout optimization.
/// For example, `Option<NonZero<u32>>` is the same size as `u32`:
///
/// ```
/// use core::{num::NonZero};
///
/// assert_eq!(size_of::<Option<NonZero<u32>>>(), size_of::<u32>());
/// ```
///
/// # Layout
///
/// `NonZero<T>` is guaranteed to have the same layout and bit validity as `T`
/// with the exception that the all-zero bit pattern is invalid.
/// `Option<NonZero<T>>` is guaranteed to be compatible with `T`, including in
/// FFI.
///
/// Thanks to the [null pointer optimization], `NonZero<T>` and
/// `Option<NonZero<T>>` are guaranteed to have the same size and alignment:
///
/// ```
/// use std::num::NonZero;
///
/// assert_eq!(size_of::<NonZero<u32>>(), size_of::<Option<NonZero<u32>>>());
/// assert_eq!(align_of::<NonZero<u32>>(), align_of::<Option<NonZero<u32>>>());
/// ```
///
/// [null pointer optimization]: crate::option#representation
///
/// # Note on generic usage
///
/// `NonZero<T>` can only be used with some standard library primitive types
/// (such as `u8`, `i32`, and etc.). The type parameter `T` must implement the
/// internal trait [`ZeroablePrimitive`], which is currently permanently unstable
/// and cannot be implemented by users. Therefore, you cannot use `NonZero<T>`
/// with your own types, nor can you implement traits for all `NonZero<T>`,
/// only for concrete types.
#[stable(feature = "generic_nonzero", since = "1.79.0")]
#[repr(transparent)]
#[rustc_nonnull_optimization_guaranteed]
#[rustc_diagnostic_item = "NonZero"]
pub struct NonZero<T: ZeroablePrimitive>(T::NonZeroInner);
