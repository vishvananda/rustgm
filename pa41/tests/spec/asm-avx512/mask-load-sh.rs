#![feature(repr_simd)]
#![feature(f16)]
// Reducer: core_arch x86/avx512fp16.rs `_mm_mask_load_sh` and the 64-bit `vpl` macro of x86/macros.rs at pinned Rust 88d9e12ae (MIT/Apache-2.0, see root NOTICE); vectors are local repr(simd) declarations with the original lanes.
use core::arch::asm;
#[repr(simd)]
#[derive(Clone,Copy)]
pub struct __m128h([f16; 8]);
pub type __mmask8 = u8;
#[cfg(target_pointer_width = "64")]
macro_rules! vpl {
    ($inst:expr) => {
        concat!($inst, ", [{p}]")
    };
}

#[inline]
#[target_feature(enable = "avx512fp16")]
pub unsafe fn _mm_mask_load_sh(src: __m128h, k: __mmask8, mem_addr: *const f16) -> __m128h {
    let mut dst = src;
    asm!(
        vpl!("vmovsh {dst}{{{k}}}"),
        dst = inout(xmm_reg) dst,
        k = in(kreg) k,
        p = in(reg) mem_addr,
        options(pure, readonly, nostack, preserves_flags)
    );
    dst
}
