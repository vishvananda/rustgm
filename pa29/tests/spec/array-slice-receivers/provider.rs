pub trait Inspect {
    fn inspect(&self) -> u8;
    fn inspect_mut(&mut self) -> u8;
}

impl Inspect for [u8] {
    fn inspect(&self) -> u8 {
        match self {
            [first, ..] => *first,
            [] => 0,
        }
    }

    fn inspect_mut(&mut self) -> u8 {
        match self {
            [first, ..] => {
                *first ^= 1;
                *first
            }
            [] => 0,
        }
    }
}

pub fn read<const N: usize>(value: &[u8; N]) -> u8 {
    value.inspect()
}

pub fn increase<const N: usize>(value: &mut [u8; N]) -> u8 {
    value.inspect_mut()
}
