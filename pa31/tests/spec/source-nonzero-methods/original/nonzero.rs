#[stable(feature = "generic_nonzero", since = "1.79.0")]
#[repr(transparent)]
#[rustc_nonnull_optimization_guaranteed]
#[rustc_diagnostic_item = "NonZero"]
pub struct NonZero<T: ZeroablePrimitive>(T::NonZeroInner);
