pub enum Order {
    First,
    Second,
}

impl Structural for Order {}
impl Equality for Order {}
impl Key for Order {}

pub struct Options {
    pub number: usize,
}

impl Structural for Options {}
impl Equality for Options {}
impl Key for Options {}

pub fn enum_mode<const MODE: Order>() -> u8 {
    19
}

pub fn record_mode<const MODE: Options>() -> u8 {
    MODE.number as u8
}
