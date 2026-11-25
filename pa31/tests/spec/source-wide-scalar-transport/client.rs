#![feature(no_core, f16, f128)]
#![no_core]
extern crate base;

const HALF_LOW: u16 = unsafe { base::transmute_unchecked::<f16, u16>(base::half_low()) };
const HALF_TINY: u16 = unsafe { base::transmute_unchecked::<f16, u16>(base::half_tiny()) };
const HALF_NEGATIVE_ZERO: u16 = unsafe {
    base::transmute_unchecked::<f16, u16>(base::half_negative_zero())
};
const QUAD_LOW: [u64; 2] = unsafe {
    base::transmute_unchecked::<f128, [u64; 2]>(base::quad_low())
};
const QUAD_PI: [u64; 2] = unsafe {
    base::transmute_unchecked::<f128, [u64; 2]>(base::quad_pi())
};
const QUAD_NEGATIVE_ZERO: [u64; 2] = unsafe {
    base::transmute_unchecked::<f128, [u64; 2]>(base::quad_negative_zero())
};

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
    HALF_LOW
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_half_tiny() -> u16 {
    HALF_TINY
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_half_negative_zero() -> u16 {
    HALF_NEGATIVE_ZERO
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_low_low() -> u64 {
    let [low, _] = QUAD_LOW;
    low
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_low_high() -> u64 {
    let [_, high] = QUAD_LOW;
    high
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_pi_low() -> u64 {
    let [low, _] = QUAD_PI;
    low
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_pi_high() -> u64 {
    let [_, high] = QUAD_PI;
    high
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_negative_zero_low() -> u64 {
    let [low, _] = QUAD_NEGATIVE_ZERO;
    low
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_negative_zero_high() -> u64 {
    let [_, high] = QUAD_NEGATIVE_ZERO;
    high
}
#[unsafe(no_mangle)]
pub extern "C" fn obs_half_generic() -> u16 { HALF_GENERIC }
#[unsafe(no_mangle)]
pub extern "C" fn obs_quad_generic() -> u64 { QUAD_GENERIC }
