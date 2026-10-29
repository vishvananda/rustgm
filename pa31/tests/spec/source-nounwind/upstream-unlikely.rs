#[unstable(feature = "core_intrinsics", issue = "none")]
#[rustc_nounwind]
#[inline(always)]
pub const fn unlikely(b: bool) -> bool {
    if b {
        cold_path();
        true
    } else {
        false
    }
}
