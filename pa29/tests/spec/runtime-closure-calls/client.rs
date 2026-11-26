extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn shared(input: u8, argument: u8) -> u8 {
    base::shared(input, argument)
}

#[unsafe(no_mangle)]
pub extern "C" fn update(input: u8) -> u8 {
    base::update(input)
}

#[unsafe(no_mangle)]
pub extern "C" fn consume(input: u8, argument: u8) -> u8 {
    base::consume(input, argument)
}
