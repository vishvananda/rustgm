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
enum ControlFlow<B,C=()>{Continue(C),Break(B)}
impl<B,C> ControlFlow<B,C>{fn break_value(self)->Option<B>{match self{ControlFlow::Break(x)=>Option::Some(x),ControlFlow::Continue(_)=>Option::None}}}
trait Try{type Output;}
impl<B,C> Try for ControlFlow<B,C>{type Output=C;}
trait Iterator{type Item;fn try_for_each<F,R>(&mut self,f:F)->R where F:FnMut(Self::Item)->R,R:Try<Output=()>{loop{}}}
struct Noise0(u8);
struct Noise1(u8);
struct Noise2(u8);
struct Noise3(u8);
struct Noise4(u8);
struct Noise5(u8);
struct Noise6(u8);
struct Noise7(u8);
struct Noise8(u8);
struct Noise9(u8);
struct Noise10(u8);
struct Noise11(u8);
struct Noise12(u8);
struct Noise13(u8);
struct Noise14(u8);
struct Noise15(u8);
struct Noise16(u8);
struct Noise17(u8);
struct Noise18(u8);
struct Noise19(u8);
struct Noise20(u8);
struct Noise21(u8);
struct Noise22(u8);
struct Noise23(u8);
struct Noise24(u8);
struct Noise25(u8);
struct Noise26(u8);
struct Noise27(u8);
struct Noise28(u8);
struct Noise29(u8);
struct Noise30(u8);
struct Noise31(u8);
struct Noise32(u8);
struct Noise33(u8);
struct Noise34(u8);
struct Noise35(u8);
struct Noise36(u8);
struct Noise37(u8);
struct Noise38(u8);
struct Noise39(u8);
struct Noise40(u8);
struct Noise41(u8);
struct Noise42(u8);
struct Noise43(u8);
struct Noise44(u8);
struct Noise45(u8);
struct Noise46(u8);
struct Noise47(u8);
struct Noise48(u8);
struct Noise49(u8);
struct Noise50(u8);
struct Noise51(u8);
struct Noise52(u8);
struct Noise53(u8);
struct Noise54(u8);
struct Noise55(u8);
struct Noise56(u8);
struct Noise57(u8);
struct Noise58(u8);
struct Noise59(u8);
struct Noise60(u8);
struct Noise61(u8);
struct Noise62(u8);
struct Noise63(u8);
struct Noise64(u8);
struct Noise65(u8);
struct Noise66(u8);
struct Noise67(u8);
struct Noise68(u8);
struct Noise69(u8);
struct Noise70(u8);
struct Noise71(u8);
struct Noise72(u8);
struct Noise73(u8);
struct Noise74(u8);
struct Noise75(u8);
struct Noise76(u8);
struct Noise77(u8);
struct Noise78(u8);
struct Noise79(u8);
struct Noise80(u8);
struct Noise81(u8);
struct Noise82(u8);
struct Noise83(u8);
struct Noise84(u8);
struct Noise85(u8);
struct Noise86(u8);
struct Noise87(u8);
struct Noise88(u8);
struct Noise89(u8);
struct Noise90(u8);
struct Noise91(u8);
struct Noise92(u8);
struct Noise93(u8);
struct Noise94(u8);
struct Noise95(u8);
struct Noise96(u8);
struct Noise97(u8);
struct Noise98(u8);
struct Noise99(u8);
struct Noise100(u8);
struct Noise101(u8);
struct Noise102(u8);
struct Noise103(u8);
struct Noise104(u8);
struct Noise105(u8);
struct Noise106(u8);
struct Noise107(u8);
struct Noise108(u8);
struct Noise109(u8);
struct Noise110(u8);
struct Noise111(u8);
struct Noise112(u8);
struct Noise113(u8);
struct Noise114(u8);
struct Noise115(u8);
struct Noise116(u8);
struct Noise117(u8);
struct Noise118(u8);
struct Noise119(u8);
struct Noise120(u8);
struct Noise121(u8);
struct Noise122(u8);
struct Noise123(u8);
struct Noise124(u8);
struct Noise125(u8);
struct Noise126(u8);
struct Noise127(u8);
fn code<I:Iterator>(i:&mut I)->Option<I::Item>{
let _=i.try_for_each(ControlFlow::Break).break_value();
let _=i.try_for_each(ControlFlow::Break).break_value();
let _=i.try_for_each(ControlFlow::Break).break_value();
let _=i.try_for_each(ControlFlow::Break).break_value();
let _=i.try_for_each(ControlFlow::Break).break_value();
let _=i.try_for_each(ControlFlow::Break).break_value();
let _=i.try_for_each(ControlFlow::Break).break_value();
i.try_for_each(ControlFlow::Break).break_value()
}
