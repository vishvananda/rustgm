extern crate base;

const _: [u8; 6] = [0; base::convert::<3>(5) as usize];

#[unsafe(no_mangle)]
pub extern "C" fn read_at_runtime(input: u8) -> u8 {
    base::convert::<4>(input)
}

#[unsafe(no_mangle)]
pub extern "C" fn read_nested() -> u8 {
    base::apply(10)
}
