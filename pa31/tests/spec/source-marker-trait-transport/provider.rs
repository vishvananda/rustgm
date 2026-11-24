#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[marker]
pub unsafe trait CloneFromCell: Clone {}

unsafe impl CloneFromCell for u8 {}

pub fn observe_marker<T: CloneFromCell>(result: u8, _: &T) -> u8 {
    result
}
