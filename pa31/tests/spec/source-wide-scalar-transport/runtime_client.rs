#![feature(no_core, f16, f128)]
#![no_core]
extern crate base;

const HALF_GENERIC: u16 = unsafe {
    base::transmute_unchecked::<f16, u16>(base::half_generic::<u8>())
};
const QUAD_GENERIC: u64 = {
    let [low, _] = unsafe {
        base::transmute_unchecked::<f128, [u64; 2]>(base::quad_generic::<u8>())
    };
    low
};

#[unsafe(no_mangle)]
pub extern "C" fn obs_half_low() -> u16 {
    unsafe { base::transmute_unchecked::<f16, u16>(base::half_low()) }
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_half_tiny() -> u16 {
    unsafe { base::transmute_unchecked::<f16, u16>(base::half_tiny()) }
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_half_negative_zero() -> u16 {
    unsafe { base::transmute_unchecked::<f16, u16>(base::half_negative_zero()) }
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_low_low() -> u64 {
    let [low, _] = unsafe { base::transmute_unchecked::<f128, [u64; 2]>(base::quad_low()) };
    low
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_low_high() -> u64 {
    let [_, high] = unsafe { base::transmute_unchecked::<f128, [u64; 2]>(base::quad_low()) };
    high
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_pi_low() -> u64 {
    let [low, _] = unsafe { base::transmute_unchecked::<f128, [u64; 2]>(base::quad_pi()) };
    low
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_pi_high() -> u64 {
    let [_, high] = unsafe { base::transmute_unchecked::<f128, [u64; 2]>(base::quad_pi()) };
    high
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_negative_zero_low() -> u64 {
    let [low, _] = unsafe { base::transmute_unchecked::<f128, [u64; 2]>(base::quad_negative_zero()) };
    low
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_negative_zero_high() -> u64 {
    let [_, high] = unsafe { base::transmute_unchecked::<f128, [u64; 2]>(base::quad_negative_zero()) };
    high
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_half_generic() -> u16 { HALF_GENERIC }
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_generic() -> u64 { QUAD_GENERIC }
