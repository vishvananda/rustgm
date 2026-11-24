
pub struct Window<'short, 'long: 'short> {
    pub short: &'short u8,
    pub long: &'long u8,
}

pub enum Choice<'short, 'long> where 'long: 'short {
    Short(&'short u8),
    Long(&'long u8),
}

impl Clone for u8 {
    fn clone(&self) -> Self { *self }
}

impl Copy for u8 {}
