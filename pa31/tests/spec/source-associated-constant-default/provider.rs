impl Clone for u8 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u8 {}

pub trait First: Copy {
    const SEED: Self;
}

pub trait Second: First {
    const DEFAULT: Self = Self::SEED;
}

impl First for u8 {
    const SEED: Self = 47;
}
impl Second for u8 {}
