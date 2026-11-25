#![feature(no_core)]
#![no_core]
extern crate base;

const CONST_SHIFTS: u32 = base::unary_and_shifts(-12, 2);
const CONST_REPEAT: u32 = base::repeat_and_index::<3>(5, 2);
const CONST_SLICE: u32 = base::slice_sum(&[2, 3, 4]);

#[unsafe(no_mangle)]
pub extern "C" fn obs_shifts() -> u32 {
    CONST_SHIFTS + base::unary_and_shifts(-12, 2)
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_ptr_union() -> u32 {
    base::pointer_and_union(11)
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_repeat() -> u32 {
    CONST_REPEAT + base::repeat_and_index::<3>(5, 1)
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_slice_text() -> u32 {
    CONST_SLICE + base::slice_sum(&[2, 3, 4]) + base::match_text("ok") + base::match_text("no")
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_casts() -> u32 {
    base::thin_and_fn_cast(&[19, 23]) + base::sealed_open()
}
