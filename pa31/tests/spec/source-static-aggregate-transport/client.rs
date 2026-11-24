#![feature(no_core)]
#![no_core]
extern crate base;

const FIRST_HEADER: u32 = {
    let [first, _] = base::HEADERS;
    first.0
};

const STATIC_HANDLE: base::Handle = base::handle();

#[unsafe(no_mangle)]
pub extern "C" fn observed_headers() -> u32 {
    let [_, second] = &base::HEADERS;
    match (FIRST_HEADER, second.0) {
        (19, 23) => 42,
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_digit() -> u8 {
    let [_, _, third, _] = *base::digits();
    third
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_table() -> u8 {
    match base::table() {
        [_, second] => second.delta,
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_vtable() -> u8 {
    let cloned = (STATIC_HANDLE.vtable.clone)();
    (cloned.tag)()
}
