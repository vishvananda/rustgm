#![feature(repr_simd)]
// Reducer: core_arch x86/avx.rs `_mm256_stream_pd` and the 64-bit `vps` macro of x86/macros.rs at pinned Rust 88d9e12ae (MIT/Apache-2.0, see root NOTICE); the vector is a local repr(simd) declaration with the original lanes.
mod arch { pub use core::arch::asm; }
#[repr(simd)]
#[derive(Clone, Copy)]
pub struct __m256d([f64; 4]);
#[cfg(target_pointer_width = "64")]
macro_rules! vps {
    ($inst1:expr, $inst2:expr) => {
        concat!($inst1, " [{p}]", $inst2)
    };
}

#[inline]
#[target_feature(enable = "avx")]
#[allow(clippy::cast_ptr_alignment)]
pub unsafe fn _mm256_stream_pd(mem_addr: *mut f64, a: __m256d) {
    // see #1541, we should use inline asm to be sure, because LangRef isn't clear enough
    crate::arch::asm!(
        vps!("vmovntpd", ",{a}"),
        p = in(reg) mem_addr,
        a = in(ymm_reg) a,
        options(nostack, preserves_flags),
    );
}
