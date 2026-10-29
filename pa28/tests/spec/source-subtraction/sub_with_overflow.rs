#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const fn sub_with_overflow<T: Copy>(x: T, y: T) -> (T, bool);
