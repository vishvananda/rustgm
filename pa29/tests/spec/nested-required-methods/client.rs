extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed(input: u8) -> u8 {
    base::nested(input)
}
