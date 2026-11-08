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
// Authored called-constructor transport controls.
// rustc's guard lowering requires `bool: Copy` under no_core.
impl Clone for bool { fn clone(&self) -> Self { *self } }
impl Copy for bool {}
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
// Authored inherent-method transport controls.
#[lang = "add"]
pub trait Add<Rhs = Self> { type Output; fn add(self, rhs: Rhs) -> Self::Output; }
impl Add for u8 { type Output = u8; fn add(self, other: u8) -> u8 { self + other } }
// A generic enum with by-value, by-reference and generic-method receivers.
pub enum Opt<T> { None, Some(T) }
impl<T> Opt<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Opt<U> { match self { Opt::Some(x) => Opt::Some(f(x)), Opt::None => Opt::None } }
    pub fn is_some(&self) -> bool { match self { Opt::Some(_) => true, Opt::None => false } }
    pub fn unwrap_or(self, default: T) -> T { match self { Opt::Some(x) => x, Opt::None => default } }
}
// A generic struct with an associated constructor, a generic impl and an
// impl on one concrete instance.
pub struct Slot<T> { pub value: T }
impl<T> Slot<T> {
    pub fn new(value: T) -> Slot<T> { Slot { value } }
    pub fn get(self) -> T { self.value }
}
impl Slot<u8> {
    pub fn peek(&self) -> u8 { self.value }
}
// A free function over a borrowed enum.
pub fn present<T>(o: &Opt<T>) -> bool { match o { Opt::Some(_) => true, Opt::None => false } }
// Authored lifetime-nominal transport controls.
// A struct with one covariant lifetime, read through free functions.
pub struct Arguments<'a> { pub text: &'a str }
pub fn take(a: Arguments<'_>) -> u8 { 42 }
pub fn first<'a>(a: Arguments<'a>, _b: Arguments<'a>) -> Arguments<'a> { a }
pub fn text<'a>(a: Arguments<'a>) -> &'a str { a.text }
// An impl on one concrete instance captures no lifetime.
impl Arguments<'static> {
    pub fn count(self) -> u8 { 42 }
}
// An invariant lifetime.
pub struct Inv<'a> { pub slot: &'a mut &'a u8 }
pub fn read(i: Inv<'_>) -> u8 { **i.slot }
// A lifetime and a type parameter; `T: 'a` follows from the field.
pub struct Ref<'a, T> { pub r: &'a T }
pub fn new_ref<'a, T>(r: &'a T) -> Ref<'a, T> { Ref { r } }
pub fn get<'a, T>(x: Ref<'a, T>) -> &'a T { x.r }
// A lifetime-generic enum with a tuple variant and a provider constructor call.
pub enum Text<'a> { Empty, Word(&'a str) }
pub fn is_word(t: &Text<'_>) -> bool { match t { Text::Word(_) => true, Text::Empty => false } }
pub fn word<'a>(s: &'a str) -> Text<'a> { Text::Word(s) }
// Authored static Sync controls: the shared lines declare no `sync` lang item.
pub fn answer() -> u8 { 42 }
