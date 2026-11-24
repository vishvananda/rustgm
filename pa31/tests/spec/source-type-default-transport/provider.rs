pub enum Flow<B, C = ()> {
    Break(B),
    Continue(C),
}

pub struct Lazy<T, F = fn() -> T> {
    pub value: T,
    pub init: F,
}

pub trait Closed<R = ()> {
    fn answer(value: R) -> u8;
}

pub trait Output {
    type Value;
}

pub trait Fallback<R = <Self as Output>::Value>: Output {
    fn answer(value: R) -> u8;
}
