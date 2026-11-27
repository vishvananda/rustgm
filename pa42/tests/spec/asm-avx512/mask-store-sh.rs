#![feature(repr_simd)]
#![feature(f16)]
// Reducer: core_arch x86/avx512fp16.rs `_mm_mask_store_sh` and the 64-bit `vps` macro of x86/macros.rs at pinned Rust 88d9e12ae (MIT/Apache-2.0, see root NOTICE); vectors are local repr(simd) declarations with the original lanes.
use core::arch::asm;
#[repr(simd)]
#[derive(Clone,Copy)]
pub struct __m128h([f16; 8]);
pub type __mmask8 = u8;
#[cfg(target_pointer_width = "64")]
macro_rules! vps {
    ($inst1:expr, $inst2:expr) => {
        concat!($inst1, " [{p}]", $inst2)
    };
}

#[inline]
#[target_feature(enable = "avx512fp16")]
pub unsafe fn _mm_mask_store_sh(mem_addr: *mut f16, k: __mmask8, a: __m128h) {
    asm!(
        vps!("vmovdqu16", "{{{k}}}, {src}"),
        p = in(reg) mem_addr,
        k = in(kreg) k,
        src = in(xmm_reg) a,
        options(nostack, preserves_flags)
    );
}
