#![feature(no_core,lang_items,rustc_attrs,fundamental,staged_api,intrinsics,declaration_constant_fixture)]
#![unstable(feature="declaration_constant_fixture",issue="none")]
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
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
const fn ctpop<T: Copy>(x: T) -> u32;
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
const fn ctlz<T: Copy>(x: T) -> u32;
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
const fn cttz<T: Copy>(x: T) -> u32;
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
const unsafe fn ctlz_nonzero<T: Copy>(x: T) -> u32;
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
const unsafe fn cttz_nonzero<T: Copy>(x: T) -> u32;
impl Clone for u8 { fn clone(&self) -> Self { *self } }
impl Copy for u8 {}
impl Clone for u16 { fn clone(&self) -> Self { *self } }
impl Copy for u16 {}
impl Clone for u32 { fn clone(&self) -> Self { *self } }
impl Copy for u32 {}
impl Clone for i64 { fn clone(&self) -> Self { *self } }
impl Copy for i64 {}

const ONES: usize = ctpop(0b1011_0110u8) as usize;
const LEADING: usize = ctlz(0x00f0u16) as usize;
const TRAILING: usize = cttz(0x0100u32) as usize;
const WIDE: usize = ctpop(0x7fff_ffff_ffff_ffffi64) as usize;
const LEADING_NONZERO: usize = unsafe { ctlz_nonzero(1u8) } as usize;
const TRAILING_NONZERO: usize = unsafe { cttz_nonzero(0x8000u16) } as usize;
const LEADING_ZERO: usize = ctlz(0u8) as usize;
const TRAILING_ZERO: usize = cttz(0u32) as usize;
fn lengths(
    a: [u8; ONES],
    b: [u8; LEADING],
    c: [u8; TRAILING],
    d: [u8; WIDE],
    e: [u8; LEADING_NONZERO],
    f: [u8; TRAILING_NONZERO],
    g: [u8; LEADING_ZERO],
    h: [u8; TRAILING_ZERO],
) -> ([u8; 5], [u8; 8], [u8; 8], [u8; 63], [u8; 7], [u8; 15], [u8; 8], [u8; 32]) {
    (a, b, c, d, e, f, g, h)
}
