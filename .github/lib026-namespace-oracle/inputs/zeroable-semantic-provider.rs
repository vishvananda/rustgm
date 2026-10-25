#![feature(impl_restriction)]
pub mod num { mod nonzero {
pub impl(self) unsafe trait ZeroablePrimitive: Sized + Copy {
    /// A type like `Self` but with a niche that includes zero.
    type NonZeroInner: Sized + Copy;
}
}
pub use nonzero::ZeroablePrimitive; }
