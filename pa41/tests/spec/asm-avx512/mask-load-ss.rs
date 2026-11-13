#![feature(repr_simd)]
// Reducer: core_arch x86/avx512f.rs `_mm_mask_load_ss` and the 64-bit `vpl` macro of x86/macros.rs at pinned Rust 88d9e12ae (MIT/Apache-2.0, see root NOTICE); vectors are local repr(simd) declarations with the original lanes.
use core::arch::asm;
#[repr(simd)]
#[derive(Clone,Copy)]
pub struct __m128([f32; 4]);
pub type __mmask8 = u8;
#[cfg(target_pointer_width = "64")]
macro_rules! vpl {
    ($inst:expr) => {
        concat!($inst, ", [{p}]")
    };
}

#[inline]
#[target_feature(enable = "avx512f")]
pub unsafe fn _mm_mask_load_ss(src: __m128, k: __mmask8, mem_addr: *const f32) -> __m128 {
    let mut dst: __m128 = src;
    asm!(
        vpl!("vmovss {dst}{{{k}}}"),
        p = in(reg) mem_addr,
        k = in(kreg) k,
        dst = inout(xmm_reg) dst,
        options(pure, readonly, nostack, preserves_flags),
    );
    dst
}
