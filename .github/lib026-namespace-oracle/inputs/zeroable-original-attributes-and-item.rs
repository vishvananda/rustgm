/// A marker trait for primitive types which can be zero.
///
/// This is an implementation detail for <code>[NonZero]\<T></code> which may disappear or be replaced at any time.
///
/// # Safety
///
/// Types implementing this trait must be primitives that are valid when zeroed.
///
/// The associated `Self::NonZeroInner` type must have the same size+align as `Self`,
/// but with a niche and bit validity making it so the following `transmutes` are sound:
///
/// - `Self::NonZeroInner` to `Option<Self::NonZeroInner>`
/// - `Option<Self::NonZeroInner>` to `Self`
///
/// (And, consequently, `Self::NonZeroInner` to `Self`.)
#[unstable(
    feature = "nonzero_internals",
    reason = "implementation detail which may disappear or be replaced at any time",
    issue = "none"
)]
pub impl(self) unsafe trait ZeroablePrimitive: Sized + Copy {
    /// A type like `Self` but with a niche that includes zero.
    type NonZeroInner: Sized + Copy;
}
