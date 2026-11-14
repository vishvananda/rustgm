/// Performs an unchecked left shift, resulting in undefined behavior when
/// `y < 0` or `y >= N`, where N is the width of T in bits.
///
/// Safe wrappers for this intrinsic are available on the integer
/// primitives via the `checked_shl` method. For example,
/// [`u32::checked_shl`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_shl<T: Copy, U: Copy>(x: T, y: U) -> T;
/// Performs an unchecked right shift, resulting in undefined behavior when
/// `y < 0` or `y >= N`, where N is the width of T in bits.
///
/// Safe wrappers for this intrinsic are available on the integer
/// primitives via the `checked_shr` method. For example,
/// [`u32::checked_shr`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_shr<T: Copy, U: Copy>(x: T, y: U) -> T;

