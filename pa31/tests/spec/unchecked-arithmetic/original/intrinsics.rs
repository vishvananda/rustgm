/// Performs an unchecked division, resulting in undefined behavior
/// where `y == 0` or `x == T::MIN && y == -1`
///
/// Safe wrappers for this intrinsic are available on the integer
/// primitives via the `checked_div` method. For example,
/// [`u32::checked_div`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_div<T: Copy>(x: T, y: T) -> T;
/// Returns the remainder of an unchecked division, resulting in
/// undefined behavior when `y == 0` or `x == T::MIN && y == -1`
///
/// Safe wrappers for this intrinsic are available on the integer
/// primitives via the `checked_rem` method. For example,
/// [`u32::checked_rem`]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_rem<T: Copy>(x: T, y: T) -> T;

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

/// Returns the result of an unchecked addition, resulting in
/// undefined behavior when `x + y > T::MAX` or `x + y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_add` on the various
/// integer types, such as [`u16::unchecked_add`] and [`i64::unchecked_add`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_add<T: Copy>(x: T, y: T) -> T;

/// Returns the result of an unchecked subtraction, resulting in
/// undefined behavior when `x - y > T::MAX` or `x - y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_sub` on the various
/// integer types, such as [`u16::unchecked_sub`] and [`i64::unchecked_sub`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_sub<T: Copy>(x: T, y: T) -> T;

/// Returns the result of an unchecked multiplication, resulting in
/// undefined behavior when `x * y > T::MAX` or `x * y < T::MIN`.
///
/// The stable counterpart of this intrinsic is `unchecked_mul` on the various
/// integer types, such as [`u16::unchecked_mul`] and [`i64::unchecked_mul`].
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_mul<T: Copy>(x: T, y: T) -> T;

