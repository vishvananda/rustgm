extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn absent() -> u8 {
    base::Marker::new().read()
}

#[unsafe(no_mangle)]
pub extern "C" fn present(value: u8) -> u8 {
    base::from_ref(&value).read()
}
