extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn shared(value: u8) -> u8 {
    base::read(&[value, 31, 48])
}

#[unsafe(no_mangle)]
pub extern "C" fn exclusive(value: u8) -> u8 {
    let mut items = [value, 71, 72, 73, 74, 75, 76];
    let updated = base::increase(&mut items);
    if items[0] == updated { updated } else { 255 }
}
