pub trait Answer {
    type Output;

    fn answer() -> u8;
    fn inherited_size() -> usize;
}

impl<T, const N: usize> Answer for [T; N] {
    type Output = u16;

    default fn answer() -> u8 {
        1
    }

    fn inherited_size() -> usize {
        N
    }
}

impl Answer for [u8; 2] {
    fn inherited_size() -> usize {
        7
    }

    fn answer() -> u8 {
        2
    }
}
