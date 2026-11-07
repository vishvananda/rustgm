#![feature(no_core,lang_items,rustc_attrs,fundamental,auto_traits)]
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
pub trait Clone: Sized {  #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"]
pub trait Copy: Clone {}


#[lang = "freeze"]
pub unsafe auto trait Freeze {}

// Oracle-only shim: rustc's checked arithmetic needs these items. RustGM never
// configures `rustc_oracle`; its consumers lower the shared abort exit.
#[cfg(rustc_oracle)]
#[lang = "panic_location"]
pub struct Location<'a> { file: &'a str, line: u32, col: u32 }
#[cfg(rustc_oracle)]
unsafe extern "C" { safe fn abort() -> !; }
#[cfg(rustc_oracle)]
#[lang = "panic_const_add_overflow"]
#[track_caller]
pub fn panic_const_add_overflow() -> ! { abort() }
#[cfg(rustc_oracle)]
#[lang = "panic_const_sub_overflow"]
#[track_caller]
pub fn panic_const_sub_overflow() -> ! { abort() }
#[cfg(rustc_oracle)]
#[lang = "panic_const_mul_overflow"]
#[track_caller]
pub fn panic_const_mul_overflow() -> ! { abort() }
// Authored operator-role transport controls.
#[lang = "add_assign"]
pub trait AddAssign<Rhs = Self> { fn add_assign(&mut self, rhs: Rhs); }
#[lang = "sub_assign"]
pub trait SubAssign<Rhs = Self> { fn sub_assign(&mut self, rhs: Rhs); }
#[lang = "mul_assign"]
pub trait MulAssign<Rhs = Self> { fn mul_assign(&mut self, rhs: Rhs); }
impl AddAssign for i32 { fn add_assign(&mut self, other: i32) { *self += other } }
impl SubAssign for i32 { fn sub_assign(&mut self, other: i32) { *self -= other } }
impl MulAssign for i32 { fn mul_assign(&mut self, other: i32) { *self *= other } }
impl Clone for i32 { fn clone(&self) -> Self { *self } }
impl Copy for i32 {}
pub struct Counted<T> { pub value: T, pub count: i32 }
pub fn bump<T: Copy>(value: T, count: i32, step: i32) -> Counted<T> { let mut total = count; total += step; Counted { value, count: total } }
pub fn lower<T: Copy>(value: T, count: i32, step: i32) -> Counted<T> { let mut total = count; total -= step; total -= step; Counted { value, count: total } }
pub fn scale<T: Copy>(value: T, count: i32, factor: i32) -> Counted<T> { let mut total = count; total *= factor; Counted { value, count: total } }
