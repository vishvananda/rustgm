#[unstable(
    feature = "nonzero_internals",
    reason = "implementation detail which may disappear or be replaced at any time",
    issue = "none"
)]
pub impl(self) unsafe trait ZeroablePrimitive: Sized + Copy {
    /// A type like `Self` but with a niche that includes zero.
    type NonZeroInner: Sized + Copy;
}
