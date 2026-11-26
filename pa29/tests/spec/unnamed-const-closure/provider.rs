pub const fn convert<const OFFSET: u8>(input: u8) -> u8 {
    const _: fn(value: u8) = |value: u8| {
        let checked = value;
        match checked {
            0 => (),
            _ => (),
        }
    };
    input ^ OFFSET
}

pub fn apply(input: u8) -> u8 {
    fn keep(value: u8) -> u8 {
        value ^ 3
    }
    keep(input)
}
