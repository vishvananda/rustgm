#![feature(no_core, asm_native_fixture)]
#![no_core]

extern crate base;

#[target_feature(enable = "avx512f")]
unsafe fn load(src: usize, address: usize, mask: u8, destination: usize) {
    unsafe {
        *(destination as *mut base::__m128) =
            base::_mm_mask_load_ss(*(src as *const base::__m128), mask, address as *const f32);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_mask(src: usize, address: usize, mask: u8, destination: usize) {
    unsafe { load(src, address, mask, destination) }
}
