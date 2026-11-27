extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn inherited_slice() -> u32 {
    <u32 as base::Inherited>::slice(&[1, 2, 3])
}

#[unsafe(no_mangle)]
pub extern "C" fn inherited_array() -> u32 {
    <u32 as base::Inherited>::array(&[1, 2])
}

#[unsafe(no_mangle)]
pub extern "C" fn scoped_slice() -> u32 {
    <u32 as base::Scoped>::slice(&[1])
}
