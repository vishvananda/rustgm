#[rustc_specialization_trait]
pub const unsafe trait Applicable<Rhs = Self> {}
const unsafe impl Applicable for u8 {}

#[rustc_unsafe_specialization_marker]
pub unsafe trait Trusted: Sized {}
unsafe impl Trusted for u8 {}

pub trait Compared {
    fn answer() -> u8;
}

impl<T, const N: usize> Compared for [T; N] {
    default fn answer() -> u8 {
        1
    }
}

impl<T: Applicable<T>, const N: usize> Compared for [T; N] {
    fn answer() -> u8 {
        2
    }
}

pub trait Ordered {
    fn answer() -> u8;
}

impl<T, const N: usize> Ordered for [T; N] {
    default fn answer() -> u8 {
        3
    }
}

impl<T: Trusted, const N: usize> Ordered for [T; N] {
    fn answer() -> u8 {
        4
    }
}
