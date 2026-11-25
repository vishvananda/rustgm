#![feature(no_core)]
#![no_core]
extern crate base;

const CONST_COUNT: u32 = base::count_three();

#[unsafe(no_mangle)]
pub extern "C" fn obs_const_count() -> u32 {
    CONST_COUNT + base::count_three()
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_reify() -> u32 {
    base::reify_closure(35)
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_capture_fn() -> u32 {
    base::capture_fn_and_adapt(10, 0u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_unique_mut() -> u32 {
    base::capture_unique_mut(5)
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_once_move() -> u32 {
    base::capture_once_move(base::Held(33))
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_nested_ref() -> u32 {
    let a = 77u32;
    let b = 88u32;
    *base::capture_ref_and_nested(&a, &b)
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_rpit_closure() -> u32 {
    let add5 = base::make_adder(5);
    add5(17)
}
