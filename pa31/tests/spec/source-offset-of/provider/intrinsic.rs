#[rustc_nounwind]
#[unstable(feature = "core_intrinsics", issue = "none")]
#[rustc_const_unstable(feature = "core_intrinsics", issue = "none")]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
#[lang = "offset_of"]
pub const fn offset_of<T: PointeeSized>(variant: u32, field: u32) -> usize;