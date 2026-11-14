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

// Authored module-constant transport controls; the declarations above are
// lines 1-57 of spec/source-branches/provider.rs, unchanged. The Copy impls
// let rustc's match lowering read the matched primitives under no_core.
impl Clone for usize { fn clone(&self) -> Self { *self } }
impl Copy for usize {}
impl Clone for bool { fn clone(&self) -> Self { *self } }
impl Copy for bool {}
impl Clone for char { fn clone(&self) -> Self { *self } }
impl Copy for char {}
impl Clone for u8 { fn clone(&self) -> Self { *self } }
impl Copy for u8 {}
impl Clone for u128 { fn clone(&self) -> Self { *self } }
impl Copy for u128 {}
pub const LANES: usize = 4;
pub const FLAG: bool = true;
pub const LETTER: char = 'x';
pub const NEG: i8 = 0xFF_u8 as i8;
pub const HALF: f32 = 42.5;
pub const QUARTER: f64 = 42.25;
pub const BIG: u128 = 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF;
pub const NOTHING: () = ();
const SECRET: u8 = 42;
pub fn lanes() -> usize { LANES }
pub fn neg() -> i8 { NEG }
pub fn secret() -> u8 { SECRET }
pub fn nested() -> u8 { const INNER: u8 = 42; INNER }
pub fn first_inner() -> u8 { const INNER: u8 = 40; INNER }
pub fn second_inner() -> u8 { const INNER: u8 = 2; INNER }
pub const fn lane_count() -> usize { LANES }

#[lang="future_trait"]pub trait Future{#[lang="future_output"]type Output;}
#[lang = "async_drop_in_place"]
pub async unsafe fn cleanup<T: ?Sized>(_to_drop: &mut T) {
    // Code here does not matter - this is replaced by the
    // real implementation by the compiler.
}

