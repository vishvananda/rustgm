#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

fn pick(x: &u8) -> u8 {
    match *x {
        20 => 23,
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_context() -> u8 {
    let value = 41_u8;
    let cx = base::Context::new(&value);
    cx.read()
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_hooks() -> u8 {
    let value = 6_u8;
    base::run_hook(base::default_hooks(), &value)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_borrowed_fn() -> u8 {
    let value = 20_u8;
    base::apply_borrowed(pick, &value)
}
