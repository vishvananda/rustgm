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
// Authored fn-item reification controls.
impl Clone for u16 { fn clone(&self) -> Self { *self } }
impl Copy for u16 {}
pub fn hold(_v: u8) -> i32 { 42 }
pub fn take() -> fn(u8) -> i32 { hold }
pub fn id<T>(x: T) -> T { x }
pub fn pick() -> fn(u16) -> u16 { id::<u16> }
pub fn erase<T: Copy>() -> fn(T) -> T { id::<T> }
pub struct W(pub u8);
impl W { pub fn get(self) -> u8 { self.0 } }
pub fn getter() -> fn(W) -> u8 { W::get }
pub trait Tr { fn f(self) -> u8; }
impl Tr for u8 { fn f(self) -> u8 { match self { 41 => 42, _ => 7 } } }
pub fn method() -> fn(u8) -> u8 { <u8 as Tr>::f }
pub trait Show { fn show(self) -> u8; }
impl Show for u8 { fn show(self) -> u8 { self } }
pub const fn make<T: Show>() -> fn(T) -> u8 { <T as Show>::show }
pub fn as_unsafe() -> unsafe fn(u8) -> i32 { hold }
pub unsafe fn raw(_v: u8) -> i32 { 42 }
pub fn raw_ptr() -> unsafe fn(u8) -> i32 { raw }
// Authored fn-pointer call controls.
pub fn call_ptr(f: fn(u8) -> i32) -> i32 { f(7) }
pub fn twice(f: fn(u8) -> u8, v: u8) -> u8 { f(f(v)) }
pub fn apply_ptr<T>(f: fn(T) -> T, v: T) -> T { f(v) }
pub fn reified_call() -> i32 { let f: fn(u8) -> i32 = hold; f(7) }
pub fn block_call(f: unsafe fn(u8) -> i32) -> i32 { unsafe { f(7) } }
// A safe callee inside an unsafe block records no unsafe obligation.
pub fn safe_block(f: fn(u8) -> i32) -> i32 { unsafe { f(7) } }
pub unsafe fn nested_call(f: unsafe fn(u8) -> i32) -> i32 { unsafe { f(7) } }
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn bare_call(f: unsafe fn(u8) -> i32) -> i32 { f(7) }
pub struct Runner;
impl Runner { pub fn run(f: fn(u8) -> i32) -> i32 { f(7) } }
// Core's Argument::fmt calls the formatter it stores in a record variant;
// this twin receives it as a parameter.
pub unsafe fn render(value: u8, f: u8) -> i32 { match value { 20 => match f { 22 => 42, _ => 7 }, _ => 7 } }
pub unsafe fn fmt(value: u8, f: u8, formatter: unsafe fn(u8, u8) -> i32) -> i32 { unsafe { formatter(value, f) } }
// Authored constructor reification controls.
pub struct P(pub u8);
pub fn ctor() -> fn(u8) -> P { P }
pub enum E { A(u8), B }
pub fn variant() -> fn(u8) -> E { E::A }
pub enum Opt<T> { Some(T), None }
pub fn some<T>() -> fn(T) -> Opt<T> { Opt::Some }
// Authored nominal callable field controls, shaped as core's task/wake.rs
// RawWakerVTable, io/error.rs OsFunctions and marker/variance.rs markers.
#[lang = "phantom_data"]
pub struct PhantomData<T: PointeeSized>;
#[rustc_diagnostic_item = "Send"]
pub unsafe auto trait Send {}
#[lang = "sync"]
pub unsafe auto trait Sync {}
pub struct Slot { pub f: fn(u8) -> i32 }
pub fn via(s: &Slot) -> i32 { (s.f)(7) }
pub fn slot_new() -> Slot { Slot { f: hold } }
pub fn store(s: &mut Slot, f: fn(u8) -> i32) { s.f = f; }
impl Slot { pub fn call(&self) -> i32 { (self.f)(7) } }
pub fn held() -> i32 { hold(7) }
pub struct Table { pub call: unsafe fn(*const ()) -> i32, pub drop: unsafe fn(*const ()) }
pub unsafe fn run(t: &Table, p: *const ()) -> i32 { unsafe { (t.call)(p) } }
pub struct Op(pub fn(u8, u8) -> i32);
pub fn op_apply(o: &Op, a: u8, b: u8) -> i32 { (o.0)(a, b) }
fn add3(a: u8, b: u8) -> i32 { match a { 40 => match b { 2 => 42, _ => 7 }, _ => 7 } }
pub fn op_new() -> Op { Op(add3) }
pub enum Choice { Fixed(u8), Computed(fn(u8) -> i32) }
pub fn choose(c: &Choice, x: u8) -> i32 { match *c { Choice::Computed(f) => f(x), Choice::Fixed(_) => 7 } }
fn inc3(x: u8) -> i32 { match x { 41 => 42, _ => 7 } }
pub fn computed() -> Choice { Choice::Computed(inc3) }
pub struct Apply<T> { pub f: fn(T) -> T }
pub fn apply_field<T>(a: &Apply<T>, x: T) -> T { (a.f)(x) }
pub struct Contra<T>(pub PhantomData<fn(T)>);
pub fn contra<T>() -> Contra<T> { Contra(PhantomData) }
pub struct Inv<T: ?Sized>(pub PhantomData<fn(T) -> T>);
pub fn inv_new<T: ?Sized>() -> Inv<T> { Inv(PhantomData) }
// derive(Clone, Copy)'s expansion for a non-generic Copy struct.
pub struct Pair { pub f: fn(u8) -> i32, pub v: u8 }
impl Clone for Pair { fn clone(&self) -> Self { *self } }
impl Copy for Pair {}
