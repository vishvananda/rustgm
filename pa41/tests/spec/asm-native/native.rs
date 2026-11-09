#![feature(repr_simd,simd_ffi)]
#![feature(no_core,lang_items,rustc_attrs,fundamental,staged_api,const_trait_impl,const_destruct,decl_macro)]
#![unstable(feature="asm_native_fixture",issue="none")]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang = "pointee_sized"]
#[diagnostic::on_unimplemented(
    message = "values of type `{Self}` may or may not have a size",
    label = "may or may not have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_coinductive]
pub trait PointeeSized {
    // Empty
}
#[lang = "meta_sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known",
    label = "doesn't have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
// `MetaSized` being coinductive, despite having supertraits, is okay for the same reasons as
// `Sized` above.
#[rustc_coinductive]
pub trait MetaSized: PointeeSized {
    // Empty
}
#[lang = "sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known at compilation time",
    label = "doesn't have a size known at compile-time"
)]
#[fundamental] // for Default, for example, which requires that `[T]: !Default` be evaluatable
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
// `Sized` being coinductive, despite having supertraits, is okay as there are no user-written impls,
// and we know that the supertraits are always implemented if the subtrait is just by looking at
// the builtin impls.
#[rustc_coinductive]
pub trait Sized: MetaSized {
    // Empty.
}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "clone"]
pub trait Clone: Sized { #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"]
pub trait Copy: Clone {}
#[unstable(feature="const_destruct",issue="133214")]
#[rustc_const_unstable(feature="const_destruct",issue="133214")]
#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub const trait Destruct:PointeeSized{}
impl<T> Clone for *mut T{fn clone(&self)->Self{*self}}impl<T> Copy for *mut T{}
impl Clone for f32{fn clone(&self)->Self{*self}}impl Copy for f32{}
// The source-owned `core::arch::asm` declaration and the builtin `concat!`.
pub mod arch {
    #[stable(feature = "asm", since = "1.59.0")]
    #[rustc_builtin_macro]
    pub macro asm("assembly template", $(operands,)* $(options($(option),*))?) {}
}
#[rustc_builtin_macro(concat)]
macro_rules! concat { ($($t:tt)*) => {}; }
#[repr(simd)]
pub struct __m128([f32;4]);
impl<T:Copy,const N:usize>Clone for [T;N]{fn clone(&self)->Self{*self}}impl<T:Copy,const N:usize>Copy for [T;N]{}
impl Clone for __m128{fn clone(&self)->Self{*self}}impl Copy for __m128{}
// Pinned stdarch x86/macros.rs and x86/sse.rs bytes.
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
// Authored observers. Lanes come from C as bit-exact f32 returns.
unsafe extern "C" {
    fn lane(index: u32) -> f32;
    fn rg_observe();
}
fn low() -> __m128 {
    unsafe { __m128([lane(0), lane(1), lane(2), lane(3)]) }
}
fn high() -> __m128 {
    unsafe { __m128([lane(4), lane(5), lane(6), lane(7)]) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rg_stream_ps(p: *mut f32, a: __m128) {
    unsafe { _mm_stream_ps(p, a) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rg_stream_lanes(p: *mut f32) {
    unsafe { _mm_stream_ps(p, low()) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rg_stream_default(p: *mut f32, a: __m128) {
    unsafe { crate::arch::asm!("movntps [{p}],{a}", p = in(reg) p, a = in(xmm_reg) a) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rg_reassigned(p: *mut f32, q: *mut f32) {
    let mut a = low();
    unsafe {
        crate::arch::asm!("movntps [{p}],{a}", p = in(reg) p, a = in(xmm_reg) a, options(nostack, preserves_flags));
    }
    a = high();
    unsafe { _mm_stream_ps(q, a) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rg_operand_order(p: *mut f32, q: *mut f32) {
    let mut a = low();
    unsafe {
        crate::arch::asm!(
            "movntps [{p}],{a}",
            a = in(xmm_reg) a,
            p = in(reg) { a = high(); p },
            options(nostack, preserves_flags),
        );
    }
    unsafe { _mm_stream_ps(q, a) }
}
unsafe fn tagged<T>(p: *mut f32, a: __m128) {
    unsafe {
        crate::arch::asm!("movntps [{p}],{a}", p = in(reg) p, a = in(xmm_reg) a, options(nostack, preserves_flags));
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rg_generic(p: *mut f32, q: *mut f32, a: __m128, b: __m128) {
    unsafe {
        tagged::<u8>(p, a);
        tagged::<u32>(q, b);
    }
}
fn main() {
    unsafe { rg_observe(); }
}
