#[lang = "freeze"]
pub unsafe auto trait Freeze {}

pub const fn borrow_pointer<T: PointeeSized>(input: &T) -> *const T {
    input as *const T
}
