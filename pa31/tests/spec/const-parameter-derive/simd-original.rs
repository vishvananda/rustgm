/// A type for alignment options for SIMD masked load/store intrinsics.
#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum SimdAlign {
    // These values must match the compiler's `SimdAlign` defined in
    // `rustc_middle/src/ty/consts/int.rs`!
    /// No alignment requirements on the pointer
    Unaligned = 0,
    /// The pointer must be aligned to the element type of the SIMD vector
    Element = 1,
    /// The pointer must be aligned to the SIMD vector type
    Vector = 2,
}

