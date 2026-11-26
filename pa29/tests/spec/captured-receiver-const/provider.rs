pub struct Window<'a, const N: usize> {
    pub value: &'a u8,
}

pub trait Read {
    fn read(&self) -> u8;
}

impl<const N: usize> Read for Window<'_, N> {
    fn read(&self) -> u8 {
        match N {
            3 => *self.value,
            _ => 7,
        }
    }
}
