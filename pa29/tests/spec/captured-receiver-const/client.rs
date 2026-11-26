extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn read_three(value: u8) -> u8 {
    let window = base::Window::<3> { value: &value };
    base::Read::read(&window)
}

#[unsafe(no_mangle)]
pub extern "C" fn read_four(value: u8) -> u8 {
    let window = base::Window::<4> { value: &value };
    base::Read::read(&window)
}
