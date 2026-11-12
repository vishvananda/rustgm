#![feature(no_core,lang_items,rustc_attrs,fundamental,staged_api,const_trait_impl,const_destruct,unboxed_closures,tuple_trait,fn_traits)]
#![unstable(feature="source_fn_sugar_fixture",issue="none")]
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
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for i32{fn clone(&self)->Self{*self}}impl Copy for i32{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}

#[unstable(feature = "tuple_trait", issue = "157987")]
#[lang = "tuple_trait"]
#[diagnostic::on_unimplemented(message = "`{Self}` is not a tuple")]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub trait Tuple {}

#[lang = "fn"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_paren_sugar]
#[rustc_on_unimplemented(
    on(
        Args = "()",
        note = "wrap the `{Self}` in a closure with no arguments: `|| {{ /* code */ }}`"
    ),
    on(
        Self = "unsafe fn",
        note = "unsafe function cannot be called generically without an unsafe block",
        // SAFETY: tidy is not smart enough to tell that the below unsafe block is a string
        label = "call the function in a closure: `|| unsafe {{ /* code */ }}`"
    ),
    message = "expected an `{This:resolved}` closure, found `{Self}`",
    label = "expected an `{This:resolved}` closure, found `{Self}`"
)]
#[fundamental] // so that regex can rely that `&str: !FnMut`
#[must_use = "closures are lazy and do nothing unless called"]
#[rustc_const_unstable(feature = "const_trait_impl", issue = "143874")]
pub const trait Fn<Args: Tuple>: [const] FnMut<Args> {
    /// Performs the call operation.
    #[unstable(feature = "fn_traits", issue = "29625")]
    extern "rust-call" fn call(&self, args: Args) -> Self::Output;
}

#[lang = "fn_mut"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_paren_sugar]
#[rustc_on_unimplemented(
    on(
        Args = "()",
        note = "wrap the `{Self}` in a closure with no arguments: `|| {{ /* code */ }}`"
    ),
    on(
        Self = "unsafe fn",
        note = "unsafe function cannot be called generically without an unsafe block",
        // SAFETY: tidy is not smart enough to tell that the below unsafe block is a string
        label = "call the function in a closure: `|| unsafe {{ /* code */ }}`"
    ),
    message = "expected an `{This:resolved}` closure, found `{Self}`",
    label = "expected an `{This:resolved}` closure, found `{Self}`"
)]
#[fundamental] // so that regex can rely that `&str: !FnMut`
#[must_use = "closures are lazy and do nothing unless called"]
#[rustc_const_unstable(feature = "const_trait_impl", issue = "143874")]
pub const trait FnMut<Args: Tuple>: FnOnce<Args> {
    /// Performs the call operation.
    #[unstable(feature = "fn_traits", issue = "29625")]
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output;
}

#[lang = "fn_once"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_paren_sugar]
#[rustc_on_unimplemented(
    on(
        Args = "()",
        note = "wrap the `{Self}` in a closure with no arguments: `|| {{ /* code */ }}`"
    ),
    on(
        Self = "unsafe fn",
        note = "unsafe function cannot be called generically without an unsafe block",
        // SAFETY: tidy is not smart enough to tell that the below unsafe block is a string
        label = "call the function in a closure: `|| unsafe {{ /* code */ }}`"
    ),
    message = "expected an `{This:resolved}` closure, found `{Self}`",
    label = "expected an `{This:resolved}` closure, found `{Self}`"
)]
#[fundamental] // so that regex can rely that `&str: !FnMut`
#[must_use = "closures are lazy and do nothing unless called"]
#[rustc_const_unstable(feature = "const_trait_impl", issue = "143874")]
pub const trait FnOnce<Args: Tuple> {
    /// The returned type after the call operator is used.
    #[lang = "fn_once_output"]
    #[stable(feature = "fn_once_output", since = "1.12.0")]
    type Output;

    /// Performs the call operation.
    #[unstable(feature = "fn_traits", issue = "29625")]
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
}

enum Option<T>{None,Some(T)}
impl<T> Option<T>{
fn and_then<U,F>(self,f:F)->Option<U>where F:FnOnce(T)->Option<U>{loop{}}
fn map<U,F>(self,f:F)->Option<U>where F:FnOnce(T)->U{loop{}}
fn or(self,other:Self)->Self{self}
}
trait Iterator{type Item;fn last(self)->Option<Self::Item>where Self:Sized{loop{}}}
fn make<I:Iterator>(_x0:Option<I::Item>,_x1:Option<I::Item>,_x2:Option<I::Item>,_x3:Option<I::Item>,_x4:Option<I::Item>,_x5:Option<I::Item>,_x6:Option<I::Item>,_x7:Option<I::Item>,_x8:Option<I::Item>,_x9:Option<I::Item>,_x10:Option<I::Item>,_x11:Option<I::Item>,_x12:Option<I::Item>,_x13:Option<I::Item>,_x14:Option<I::Item>,_x15:Option<I::Item>,_x16:Option<I::Item>,_x17:Option<I::Item>,_x18:Option<I::Item>,_x19:Option<I::Item>,_x20:Option<I::Item>,_x21:Option<I::Item>,_x22:Option<I::Item>,_x23:Option<I::Item>,_x24:Option<I::Item>,_x25:Option<I::Item>,_x26:Option<I::Item>,_x27:Option<I::Item>,_x28:Option<I::Item>,_x29:Option<I::Item>,_x30:Option<I::Item>,_x31:Option<I::Item>,_iter:I)->Option<I::Item>{loop{}}
fn drive<U:Iterator,F>(f:F)->Option<U::Item>where F:FnMut(Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,U)->Option<U::Item>{loop{}}
fn code<U:Iterator>()->Option<U::Item>{drive::<U,_>(make)}
struct Noise0;
struct Noise1;
struct Noise2;
struct Noise3;
struct Noise4;
struct Noise5;
struct Noise6;
struct Noise7;
struct Noise8;
struct Noise9;
struct Noise10;
struct Noise11;
struct Noise12;
struct Noise13;
struct Noise14;
struct Noise15;
struct Noise16;
struct Noise17;
struct Noise18;
struct Noise19;
struct Noise20;
struct Noise21;
struct Noise22;
struct Noise23;
struct Noise24;
struct Noise25;
struct Noise26;
struct Noise27;
struct Noise28;
struct Noise29;
struct Noise30;
struct Noise31;
struct Noise32;
struct Noise33;
struct Noise34;
struct Noise35;
struct Noise36;
struct Noise37;
struct Noise38;
struct Noise39;
struct Noise40;
struct Noise41;
struct Noise42;
struct Noise43;
struct Noise44;
struct Noise45;
struct Noise46;
struct Noise47;
struct Noise48;
struct Noise49;
struct Noise50;
struct Noise51;
struct Noise52;
struct Noise53;
struct Noise54;
struct Noise55;
struct Noise56;
struct Noise57;
struct Noise58;
struct Noise59;
struct Noise60;
struct Noise61;
struct Noise62;
struct Noise63;
struct Noise64;
struct Noise65;
struct Noise66;
struct Noise67;
struct Noise68;
struct Noise69;
struct Noise70;
struct Noise71;
struct Noise72;
struct Noise73;
struct Noise74;
struct Noise75;
struct Noise76;
struct Noise77;
struct Noise78;
struct Noise79;
struct Noise80;
struct Noise81;
struct Noise82;
struct Noise83;
struct Noise84;
struct Noise85;
struct Noise86;
struct Noise87;
struct Noise88;
struct Noise89;
struct Noise90;
struct Noise91;
struct Noise92;
struct Noise93;
struct Noise94;
struct Noise95;
struct Noise96;
struct Noise97;
struct Noise98;
struct Noise99;
struct Noise100;
struct Noise101;
struct Noise102;
struct Noise103;
struct Noise104;
struct Noise105;
struct Noise106;
struct Noise107;
struct Noise108;
struct Noise109;
struct Noise110;
struct Noise111;
struct Noise112;
struct Noise113;
struct Noise114;
struct Noise115;
struct Noise116;
struct Noise117;
struct Noise118;
struct Noise119;
struct Noise120;
struct Noise121;
struct Noise122;
struct Noise123;
struct Noise124;
struct Noise125;
struct Noise126;
struct Noise127;
