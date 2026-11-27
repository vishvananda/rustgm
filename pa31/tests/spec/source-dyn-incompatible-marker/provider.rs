#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[rustc_dyn_incompatible_trait]
pub unsafe trait Pure: PointeeSized {}

pub struct Marked<T>(pub T);

unsafe impl<T> Pure for Marked<T> {}
unsafe impl<T> Pure for &T {}
unsafe impl<T> Pure for &mut T {}

pub fn observe<T: Pure>(_: &T, value: u8) -> u8 {
    value
}
