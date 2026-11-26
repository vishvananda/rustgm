pub enum Maybe<T> {
    Absent,
    Present(T),
}

pub struct Marker<'a>(Maybe<&'a u8>);

impl Marker<'_> {
    pub const fn new() -> Self {
        Self(Maybe::Absent)
    }

    pub fn read(&self) -> u8 {
        match self.0 {
            Maybe::Absent => 7,
            Maybe::Present(value) => *value,
        }
    }
}

pub fn from_ref<'a>(value: &'a u8) -> Marker<'a> {
    Marker(Maybe::Present(value))
}
