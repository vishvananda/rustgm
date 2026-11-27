#![feature(decl_macro,staged_api)]
#![stable(feature = "core", since = "1.6.0")]
pub mod arch {
    #[stable(feature = "asm", since = "1.59.0")]
    #[rustc_builtin_macro]
    pub macro asm("assembly template", $(operands,)* $(options($(option),*))?) {}
}
#[repr(simd)]
pub struct __m128([f32;4]);
#[cfg(target_pointer_width = "64")]
macro_rules! vps {
    ($inst1:expr, $inst2:expr) => {
        concat!($inst1, " [{p}]", $inst2)
    };
}

#[inline]
#[target_feature(enable = "sse")]
#[cfg_attr(test, assert_instr(movntps))]
#[stable(feature = "simd_x86", since = "1.27.0")]
#[allow(clippy::cast_ptr_alignment)]
pub unsafe fn _mm_stream_ps(mem_addr: *mut f32, a: __m128) {
    // see #1541, we should use inline asm to be sure, because LangRef isn't clear enough
    crate::arch::asm!(
        vps!("movntps", ",{a}"),
        p = in(reg) mem_addr,
        a = in(xmm_reg) a,
        options(nostack, preserves_flags),
    );
}
