extern crate base;

fn change(value: &mut u8) -> u8 {
    *value ^= 7;
    *value
}

#[unsafe(no_mangle)]
pub extern "C" fn plain(input: u8) -> u8 {
    base::through_parameter(input, change)
}

#[unsafe(no_mangle)]
pub extern "C" fn field(input: u8) -> u8 {
    base::through_field((input,), change)
}
