#![feature(repr_simd)]
// Reducer: core_arch x86/avx512f.rs `_mm512_stream_ps` and the 64-bit `vps` macro of x86/macros.rs at pinned Rust 88d9e12ae (MIT/Apache-2.0, see root NOTICE); vectors are local repr(simd) declarations with the original lanes.
use core::arch::asm;
#[repr(simd)]
#[derive(Clone,Copy)]
pub struct __m512([f32; 16]);
#[cfg(target_pointer_width = "64")]
macro_rules! vps {
    ($inst1:expr, $inst2:expr) => {
        concat!($inst1, " [{p}]", $inst2)
    };
}

#[inline]
#[target_feature(enable = "avx512f")]
pub unsafe fn _mm512_stream_ps(mem_addr: *mut f32, a: __m512) {
    asm!(
        vps!("vmovntps", ",{a}"),
        p = in(reg) mem_addr,
        a = in(zmm_reg) a,
        options(nostack, preserves_flags),
    );
}
