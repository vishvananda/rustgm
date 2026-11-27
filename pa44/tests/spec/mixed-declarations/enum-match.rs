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
// Authored enum-and-match transport controls.
// A fieldless enum and a constant-arm match.
pub enum Two { A, B }
pub fn first() -> Two { Two::A }
pub fn second() -> Two { Two::B }
pub fn pick(t: Two) -> u8 { match t { Two::A => 1, Two::B => 2 } }
// An explicit integer representation with explicit discriminants.
#[repr(u8)]
pub enum Tag { Low = 3, High = 39 }
pub fn low() -> Tag { Tag::Low }
pub fn high() -> Tag { Tag::High }
pub fn code(t: Tag) -> u8 { t as u8 }
// A generic payload enum. Payload values use the braced tuple-variant
// constructor; called constructor items stay outside this profile.
pub enum Opt<T> { None, Some(T) }
pub fn none<T>() -> Opt<T> { Opt::None }
pub fn some<T>(x: T) -> Opt<T> { Opt::Some { 0: x } }
pub fn or<T>(o: Opt<T>, d: T) -> T { match o { Opt::Some(x) => x, Opt::None => d } }
pub fn map<T, U, F: FnOnce(T) -> U>(o: Opt<T>, f: F) -> Opt<U> {
    match o { Opt::Some(x) => Opt::Some { 0: f(x) }, Opt::None => Opt::None }
}
pub fn peek(o: Opt<u8>) -> u8 { match o { Opt::Some(ref x) => *x, Opt::None => 0 } }

#[lang="future_trait"]pub trait Future{#[lang="future_output"]type Output;}
#[lang = "async_drop_in_place"]
pub async unsafe fn cleanup<T: ?Sized>(_to_drop: &mut T) {
    // Code here does not matter - this is replaced by the
    // real implementation by the compiler.
}

