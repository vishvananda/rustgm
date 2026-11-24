pub struct Bytes<const N: usize = 2> {
    pub bytes: [u8; N],
}

pub struct Options {
    pub number: usize,
    pub enabled: bool,
}

impl Structural for bool {}
impl Equality for bool {}
impl Key for bool {}

impl Structural for Options {}
impl Equality for Options {}
impl Key for Options {}

pub trait Choose<const OPT: Options = { Options { number: 41, enabled: true } }> {
    fn amount(_: ()) -> u8;
}
