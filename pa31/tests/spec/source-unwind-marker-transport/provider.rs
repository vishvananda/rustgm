#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[rustc_diagnostic_item = "unwind_safe_trait"]
pub auto trait Safe {}

#[rustc_diagnostic_item = "ref_unwind_safe_trait"]
pub auto trait SharedSafe {}

impl<T: PointeeSized> !Safe for &mut T {}
impl<T: PointeeSized + SharedSafe> Safe for &T {}

pub struct Shared(pub u8);

pub fn require<T: PointeeSized + Safe>(_: &T) -> u8 {
    7
}

pub fn require_shared<T: PointeeSized + SharedSafe>(_: &T) -> u8 {
    11
}
