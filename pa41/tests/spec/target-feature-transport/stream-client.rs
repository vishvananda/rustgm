#![feature(no_core, asm_native_fixture, simd_ffi)]
#![no_core]
#![allow(internal_features, dead_code, unused_variables)]
extern crate base;
use base::__m128;
// The witness client: `_mm_stream_ps` and the asm observers come from `base`.
unsafe extern "C" {
    fn lane(index: u32) -> f32;
}
fn low() -> __m128 {
    unsafe { base::rg_vector([lane(0), lane(1), lane(2), lane(3)]) }
}
fn high() -> __m128 {
    unsafe { base::rg_vector([lane(4), lane(5), lane(6), lane(7)]) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rg_stream_ps(p: *mut f32, a: __m128) {
    unsafe { base::_mm_stream_ps(p, a) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rg_stream_lanes(p: *mut f32) {
    unsafe { base::_mm_stream_ps(p, low()) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rg_stream_default(p: *mut f32, a: __m128) {
    unsafe { base::stream_default(p, a) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rg_reassigned(p: *mut f32, q: *mut f32) {
    unsafe { base::reassigned(p, q, low(), high()) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rg_operand_order(p: *mut f32, q: *mut f32) {
    unsafe { base::operand_order(p, q, low(), high()) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rg_generic(p: *mut f32, q: *mut f32, a: __m128, b: __m128) {
    unsafe {
        base::tagged::<u8>(p, a);
        base::tagged::<u32>(q, b);
    }
}
