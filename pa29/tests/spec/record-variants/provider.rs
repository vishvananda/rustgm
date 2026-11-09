#![feature(no_core,lang_items,rustc_attrs,fundamental,auto_traits,unboxed_closures,const_trait_impl)]
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
// Oracle-only shim: rustc's drop elaboration of the owned FnMut/Fn argument
// needs this item. RustGM never configures `rustc_oracle`.
#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}
// Authored callable-role transport controls.
#[lang = "tuple_trait"]
#[rustc_deny_explicit_impl]
pub trait Tuple {}
#[lang = "fn_once"]
#[rustc_paren_sugar]
pub const trait FnOnce<Args: Tuple> {
    #[lang = "fn_once_output"]
    type Output;
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
}
#[lang = "fn_mut"]
#[rustc_paren_sugar]
pub const trait FnMut<Args: Tuple>: FnOnce<Args> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output;
}
#[lang = "fn"]
#[rustc_paren_sugar]
pub const trait Fn<Args: Tuple>: [const] FnMut<Args> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output;
}
impl Clone for u8 { fn clone(&self) -> Self { *self } }
impl Copy for u8 {}
pub fn apply<F: FnOnce(u8) -> u8>(f: F, x: u8) -> u8 { f(x) }
pub fn apply_mut<F: FnMut(u8) -> u8>(mut f: F, x: u8) -> u8 { f(x) }
pub fn apply_ref<F: Fn(u8) -> u8>(f: F, x: u8) -> u8 { f(x) }
// Authored record-variant transport controls.
impl Clone for usize { fn clone(&self) -> Self { *self } }
impl Copy for usize {}
// Record, tuple, unit and empty record variants in one enum.
pub enum Shape { Unit, Pair(u8, u8), Rect { w: u8, h: u8 }, Empty {} }
pub fn rect(w: u8, h: u8) -> Shape { Shape::Rect { h, w } }
pub fn pair(a: u8, b: u8) -> Shape { Shape::Pair(a, b) }
pub fn empty() -> Shape { Shape::Empty {} }
pub fn width(s: Shape) -> u8 {
    match s { Shape::Rect { w, .. } => w, Shape::Pair(a, _) => a, Shape::Unit => 0, Shape::Empty {} => 1 }
}
pub fn height(s: Shape) -> u8 { match s { Shape::Rect { h: x, w: _ } => x, _ => 0 } }
// core 1.98's ffi::c_str::FromBytesWithNulError shape, with the Clone and
// Copy impls its derives expand to.
pub enum NulError { InteriorNul { position: usize }, NotNulTerminated }
impl Clone for NulError { fn clone(&self) -> NulError { *self } }
impl Copy for NulError {}
pub fn interior(position: usize) -> NulError { NulError::InteriorNul { position } }
pub fn position(e: NulError) -> usize {
    match e { NulError::InteriorNul { position } => position, NulError::NotNulTerminated => 0 }
}
