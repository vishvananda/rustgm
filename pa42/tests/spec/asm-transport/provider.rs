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
// Authored asm transport section.
pub unsafe fn store(p: *mut u32) {
    unsafe {
        crate::arch::asm!("mov dword ptr [{p}], 40", p = in(reg) p, options(nostack, preserves_flags));
    }
}
pub unsafe fn bump(p: *mut u32) {
    crate::arch::asm!("add dword ptr [{p}], 1", "add dword ptr [{p}], 1", p = in(reg) p, options(nostack));
}
pub fn tagged<T>(p: *mut u32) {
    unsafe { crate::arch::asm!("# {p}", p = in(reg) p) }
}
