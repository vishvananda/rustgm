#![feature(negative_impls)]
#![feature(core_intrinsics,coverage_attribute,multiple_supertrait_upcastable,unsize,coerce_unsized,dispatch_from_dyn)]
#![feature(decl_macro,allow_internal_unstable,min_adt_const_params,const_param_ty_trait,structural_match,auto_traits,const_precise_live_drops,source_operator_fixture,no_core,lang_items,rustc_attrs,fundamental,staged_api,const_trait_impl,const_destruct,intrinsics)]
#![unstable(feature="source_operator_fixture",issue="none")]
#![no_core]
#![allow(internal_features,dead_code,unused_variables,unused_imports)]
#[lang="freeze"]pub unsafe auto trait Freeze{}
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


impl Clone for u16{fn clone(&self)->Self{*self}}impl Copy for u16{}
impl Clone for i8{fn clone(&self)->Self{*self}}impl Copy for i8{}
impl Clone for isize{fn clone(&self)->Self{*self}}impl Copy for isize{}
macro_rules! forward_ref_unop {
    (impl $imp:ident, $method:ident for $t:ty, $(#[$attr:meta])+) => {
        $(#[$attr])+
        const impl $imp for &$t {
            type Output = <$t as $imp>::Output;

            #[inline]
            fn $method(self) -> <$t as $imp>::Output {
                $imp::$method(*self)
            }
        }
    }
}

// implements binary operators "&T op U", "T op &U", "&T op &U"
// based on "T op U" where T and U are expected to be `Copy`able
#[lang = "not"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[doc(alias = "!")]
pub const trait Not {
    /// The resulting type after applying the `!` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the unary `!` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(!true, false);
    /// assert_eq!(!false, true);
    /// assert_eq!(!1u8, 254);
    /// assert_eq!(!0u8, 255);
    /// ```
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn not(self) -> Self::Output;
}
macro_rules! not_impl {
    ($($t:ty)*) => ($(
        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")]
        const impl Not for $t {
            type Output = $t;

            #[inline]
            fn not(self) -> $t { !self }
        }

        forward_ref_unop! { impl Not, not for $t,
        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")] }
    )*)
}

not_impl!{bool}
#[lang = "eq"]
#[stable(feature = "rust1", since = "1.0.0")]
#[doc(alias = "==")]
#[doc(alias = "!=")]
#[diagnostic::on_unimplemented(
    message = "can't compare `{Self}` with `{Rhs}`",
    label = "no implementation for `{Self} == {Rhs}`"
)]
#[rustc_diagnostic_item = "PartialEq"]
#[rustc_const_unstable(feature = "const_cmp", issue = "143800")]
pub const trait PartialEq<Rhs: PointeeSized = Self>: PointeeSized {
    /// Tests for `self` and `other` values to be equal, and is used by `==`.
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_diagnostic_item = "cmp_partialeq_eq"]
    fn eq(&self, other: &Rhs) -> bool;

    /// Tests for `!=`. The default implementation is almost always sufficient,
    /// and should not be overridden without very good reason.
    #[inline]
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_diagnostic_item = "cmp_partialeq_ne"]
    fn ne(&self, other: &Rhs) -> bool {
        !self.eq(other)
    }
}
    macro_rules! partial_eq_impl {
        ($($t:ty)*) => ($(
            #[stable(feature = "rust1", since = "1.0.0")]
            #[rustc_const_unstable(feature = "const_cmp", issue = "143800")]
            const impl PartialEq for $t {
                #[inline]
                fn eq(&self, other: &Self) -> bool { *self == *other }
                #[inline]
                fn ne(&self, other: &Self) -> bool { *self != *other }
            }
        )*)
    }

partial_eq_impl!{usize u16 u8 i8 bool isize}
// Exact builtin macro declaration intervals from pinned Rust88d9; module placement below is authored.
#[rustc_builtin_macro]
    pub macro derive($item:item) {
        /* compiler built-in */
    }
#[unstable(feature = "derive_const", issue = "118304")]
    #[rustc_builtin_macro]
    pub macro derive_const($item:item) {
        /* compiler built-in */
    }
#[rustc_builtin_macro]
#[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
#[allow_internal_unstable(core_intrinsics, derive_clone_copy_internals)]
pub macro Copy($item:item) {
    /* compiler built-in */
}
#[rustc_builtin_macro]
#[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
#[allow_internal_unstable(core_intrinsics, derive_clone_copy_internals, trivial_clone)]
pub macro Clone($item:item) {
    /* compiler built-in */
}
#[rustc_builtin_macro]
    #[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
    #[allow_internal_unstable(core_intrinsics, fmt_helpers_for_derive)]
    pub macro Debug($item:item) {
        /* compiler built-in */
    }
#[rustc_builtin_macro(Default, attributes(default))]
#[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
#[allow_internal_unstable(core_intrinsics)]
pub macro Default($item:item) {
    /* compiler built-in */
}


#[rustc_builtin_macro]
#[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
#[allow_internal_unstable(core_intrinsics, derive_eq_internals, structural_match)]
#[allow_internal_unstable(coverage_attribute)]
pub macro Eq($item:item) {
    /* compiler built-in */
}
#[rustc_builtin_macro]
#[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
#[allow_internal_unstable(core_intrinsics, structural_match)]
pub macro PartialEq($item:item) {
    /* compiler built-in */
}
#[rustc_builtin_macro]
    #[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
    #[allow_internal_unstable(core_intrinsics)]
    pub macro Hash($item:item) {
        /* compiler built-in */
    }
#[rustc_builtin_macro]
#[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
#[allow_internal_unstable(core_intrinsics)]
pub macro Ord($item:item) {
    /* compiler built-in */
}
#[rustc_builtin_macro]
#[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
#[allow_internal_unstable(core_intrinsics)]
pub macro PartialOrd($item:item) {
    /* compiler built-in */
}

#[unstable(feature="unsize",issue="18598")]
#[lang="unsize"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub trait Unsize<T:PointeeSized>:PointeeSized {}
#[unstable(feature="coerce_unsized",issue="18598")]
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized {}
#[unstable(feature="coerce_unsized",issue="18598")]
impl<'a,T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<&'a mut U> for &'a mut T {}
#[unstable(feature = "dispatch_from_dyn", issue = "none")]
#[lang = "dispatch_from_dyn"]
pub trait DispatchFromDyn<T>: Sized {
    // Empty.
}

// &T -> &U
#[unstable(feature = "dispatch_from_dyn", issue = "none")]
impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<&'a U> for &'a T {}
// &mut T -> &mut U
#[unstable(feature = "dispatch_from_dyn", issue = "none")]
impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<&'a mut U> for &'a mut T {}

#[lang="discriminant_kind"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub trait DiscriminantKind{#[lang="discriminant_type"]type Discriminant:Sized;}
pub mod intrinsics{
#[rustc_intrinsic]pub const unsafe fn unreachable()->!;
 use super::DiscriminantKind;
 #[unstable(feature="core_intrinsics",issue="none")]#[rustc_intrinsic_const_stable_indirect]#[rustc_nounwind]#[rustc_intrinsic]
 pub const fn discriminant_value<T>(v:&T)-><T as DiscriminantKind>::Discriminant;
}
extern crate self as core;
pub mod cmp {pub use crate::PartialEq;use crate::PointeeSized;
#[lang="Ordering"]pub enum Ordering{Less=-1,Equal=0,Greater=1}
#[rustc_diagnostic_item="PartialOrd"]pub trait PartialOrd<Rhs:PointeeSized=Self>:PartialEq<Rhs>{fn partial_cmp(&self,other:&Rhs)->crate::option::Option<Ordering>;}
#[rustc_diagnostic_item="Ord"]pub trait Ord:Eq+PartialOrd<Self>{fn cmp(&self,other:&Self)->Ordering;}
pub struct AssertParamIsEq<T:Eq>{value:crate::marker::PhantomData<T>}
impl Eq for isize{}
impl PartialOrd for isize{fn partial_cmp(&self,other:&Self)->crate::option::Option<Ordering>{loop{}}}
impl Ord for isize{fn cmp(&self,other:&Self)->Ordering{loop{}}}

#[doc(alias = "==")]
#[doc(alias = "!=")]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_diagnostic_item = "Eq"]
#[rustc_const_unstable(feature = "const_cmp", issue = "143800")]
pub const trait Eq: [const] PartialEq<Self> + PointeeSized {
    // This method was used solely by `#[derive(Eq)]` to assert that every component of a
    // type implements `Eq` itself.
    //
    // This should never be implemented by hand.
    #[doc(hidden)]
    #[coverage(off)]
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_diagnostic_item = "assert_receiver_is_total_eq"]
    #[deprecated(since = "1.95.0", note = "implementation detail of `#[derive(Eq)]`")]
    fn assert_receiver_is_total_eq(&self) {}

    // FIXME (#152504): this method is used solely by `#[derive(Eq)]` to assert that
    // every component of a type implements `Eq` itself. It will be removed again soon.
    #[doc(hidden)]
    #[coverage(off)]
    #[unstable(feature = "derive_eq_internals", issue = "none")]
    fn assert_fields_are_eq(&self) {}
}


}
pub mod marker {pub use crate::Copy;use crate::cmp::Eq;
#[lang="phantom_data"]pub struct PhantomData<T:crate::PointeeSized>;
#[unstable(feature = "structural_match", issue = "31434")]
#[diagnostic::on_unimplemented(message = "the type `{Self}` does not `#[derive(PartialEq)]`")]
#[lang = "structural_peq"]
pub trait StructuralPartialEq {
    // Empty.
}

#[lang = "const_param_ty"]
#[unstable(feature = "const_param_ty_trait", issue = "95174", implied_by = "unsized_const_params")]
#[diagnostic::on_unimplemented(message = "`{Self}` can't be used as a const parameter type")]
#[allow(multiple_supertrait_upcastable)]
// We name this differently than the derive macro so that the `adt_const_params` can
// be used independently of `unsized_const_params` without requiring a full path
// to the derive macro every time it is used. This should be renamed on stabilization.
pub trait ConstParamTy_: StructuralPartialEq + Eq {}

/// Derive macro generating an impl of the trait `ConstParamTy`.
#[rustc_builtin_macro]
#[allow_internal_unstable(const_param_ty_trait)]
#[unstable(feature = "min_adt_const_params", issue = "154042", implied_by = "adt_const_params")]
pub macro ConstParamTy($item:item) {
    /* compiler built-in */
}

// Exact actual marker.rs policy anchor for the implied feature.
#[unstable(feature = "adt_const_params", issue = "95174")]
const _: () = ();
// Authored reducer-only policy inventory anchor; grants no caller assumption.
// Original core owns this policy on its str/slice/reference implementation family.
#[unstable(feature = "unsized_const_params", issue = "95174")]
const _: () = ();

}
pub mod fmt {use crate::PointeeSized;
// Authored dependency support only; no original Formatter runtime claim.
pub struct Error;pub enum Outcome<T,E>{Ok(T),Err(E)}
pub type Result=Outcome<(),Error>;
pub trait Write{fn write_str(&mut self,s:&str)->Result;}
pub struct FormattingOptions{flags:u32,width:u16,precision:u16}
#[allow(missing_debug_implementations)]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_diagnostic_item = "Formatter"]
pub struct Formatter<'a> {
    options: FormattingOptions,

    buf: &'a mut (dyn Write + 'a),
}

    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_diagnostic_item = "Debug"]
    #[rustc_trivial_field_reads]
    pub trait Debug: PointeeSized {
        #[stable(feature = "rust1", since = "1.0.0")]
        fn fmt(&self, f: &mut Formatter<'_>) -> Result;
    }
impl<T:PointeeSized+Debug>Debug for &T{fn fmt(&self,f:&mut Formatter<'_>)->Result{T::fmt(*self,f)}}
impl Formatter<'_>{
pub fn debug_tuple_field1_finish(&mut self,name:&str,field:&dyn Debug)->Result{loop{}}
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn write_str(&mut self, data: &str) -> Result {
        self.buf.write_str(data)
    }
}
}


pub mod option{pub enum Option<T>{None,Some(T)}}
pub mod clone{pub use crate::Clone;pub struct AssertParamIsClone<T:Clone>{value:crate::marker::PhantomData<T>}pub struct AssertParamIsCopy<T:crate::Copy>{value:crate::marker::PhantomData<T>}}
pub mod hash{
use crate::PointeeSized;
#[rustc_diagnostic_item="Hasher"]pub trait Hasher{fn finish(&self)->u64;fn write(&mut self,bytes:&[u8]);}
#[rustc_diagnostic_item="Hash"]pub trait Hash:PointeeSized{fn hash<H:Hasher>(&self,state:&mut H);}
impl Hash for isize{fn hash<H:Hasher>(&self,state:&mut H){}}
}
impl<T:PointeeSized>Clone for &T{fn clone(&self)->Self{*self}}impl<T:PointeeSized>Copy for &T{}
#[lang="unpin"]pub auto trait Unpin{}
impl<T:PointeeSized>Unpin for &T{}
impl<T:PointeeSized>Unpin for &mut T{}
impl<A:PointeeSized+PartialEq<B>,B:PointeeSized>PartialEq<&B> for &A{fn eq(&self,other:&&B)->bool{A::eq(*self,*other)}}
impl<'a,T:PointeeSized+Unsize<U>,U:PointeeSized>CoerceUnsized<&'a U> for &'a T{}

pub mod ops{
#[lang="neg"]pub trait Neg{type Output;fn neg(self)->Self::Output;}
impl Neg for isize{type Output=isize;fn neg(self)->isize{-self}}

#[lang="deref"]pub trait Deref:PointeeSized{#[lang="deref_target"]type Target:PointeeSized;fn deref(&self)->&Self::Target;}
#[lang="deref_mut"]pub trait DerefMut:Deref{fn deref_mut(&mut self)->&mut Self::Target;}
use crate::{PointeeSized,LegacyReceiver};
impl<T:PointeeSized>Deref for &T{type Target=T;fn deref(&self)->&T{*self}}
impl<T:PointeeSized>Deref for &mut T{type Target=T;fn deref(&self)->&T{&**self}}
impl<T:PointeeSized>DerefMut for &mut T{fn deref_mut(&mut self)->&mut T{&mut **self}}
}
pub mod pin{
use crate::{ops::{Deref,DerefMut},PointeeSized,Unpin,LegacyReceiver};
#[lang="pin"]#[repr(transparent)]pub struct Pin<P>{pub pointer:P}
impl<P>Pin<P>{#[lang="new_unchecked"]pub const unsafe fn new_unchecked(pointer:P)->Self{Self{pointer}}}
impl<P:Deref>Pin<P>{pub fn new(pointer:P)->Self where P::Target:Unpin{Self{pointer}}}
impl<P:Deref>Deref for Pin<P>{type Target=P::Target;fn deref(&self)->&Self::Target{self.pointer.deref()}}
impl<P:DerefMut>DerefMut for Pin<P> where P::Target:Unpin{fn deref_mut(&mut self)->&mut Self::Target{self.pointer.deref_mut()}}
impl<P:DerefMut>Pin<P>{pub fn as_mut(&mut self)->Pin<&mut P::Target>{unsafe{Pin::new_unchecked(self.pointer.deref_mut())}}pub fn as_deref_mut(self:Pin<&mut Self>)->Pin<&mut P::Target>{unsafe{Pin::new_unchecked(self.pointer.pointer.deref_mut())}}}
impl<P:Deref>LegacyReceiver for Pin<P>{}
}

pub mod actual{use crate::{derive,Sized,Clone,Copy,PartialEq,PartialOrd,Eq,Ord,Debug,Hash,Unpin};
use crate::pin::Pin;

/// The result of a coroutine resumption.
///
/// This enum is returned from the `Coroutine::resume` method and indicates the
/// possible return values of a coroutine. Currently this corresponds to either
/// a suspension point (`Yielded`) or a termination point (`Complete`).
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
#[lang = "coroutine_state"]
#[unstable(feature = "coroutine_trait", issue = "43122")]
pub enum CoroutineState<Y, R> {
    /// The coroutine suspended with a value.
    ///
    /// This state indicates that a coroutine has been suspended, and typically
    /// corresponds to a `yield` statement. The value provided in this variant
    /// corresponds to the expression passed to `yield` and allows coroutines to
    /// provide a value each time they yield.
    Yielded(Y),

    /// The coroutine completed with a return value.
    ///
    /// This state indicates that a coroutine has finished execution with the
    /// provided value. Once a coroutine has returned `Complete` it is
    /// considered a programmer error to call `resume` again.
    Complete(R),
}

/// The trait implemented by builtin coroutine types.
///
/// Coroutines are currently an
/// experimental language feature in Rust. Added in [RFC 2033] coroutines are
/// currently intended to primarily provide a building block for async/await
/// syntax but will likely extend to also providing an ergonomic definition for
/// iterators and other primitives.
///
/// The syntax and semantics for coroutines is unstable and will require a
/// further RFC for stabilization. At this time, though, the syntax is
/// closure-like:
///
/// ```rust
/// #![feature(coroutines)]
/// #![feature(coroutine_trait)]
/// #![feature(stmt_expr_attributes)]
///
/// use std::ops::{Coroutine, CoroutineState};
/// use std::pin::Pin;
///
/// fn main() {
///     let mut coroutine = #[coroutine] || {
///         yield 1;
///         "foo"
///     };
///
///     match Pin::new(&mut coroutine).resume(()) {
///         CoroutineState::Yielded(1) => {}
///         _ => panic!("unexpected return from resume"),
///     }
///     match Pin::new(&mut coroutine).resume(()) {
///         CoroutineState::Complete("foo") => {}
///         _ => panic!("unexpected return from resume"),
///     }
/// }
/// ```
///
/// More documentation of coroutines can be found in the [unstable book].
///
/// [RFC 2033]: https://github.com/rust-lang/rfcs/pull/2033
/// [unstable book]: ../../unstable-book/language-features/coroutines.html
#[lang = "coroutine"]
#[unstable(feature = "coroutine_trait", issue = "43122")]
#[fundamental]
#[must_use = "coroutines are lazy and do nothing unless resumed"]
pub trait Coroutine<R = ()> {
    /// The type of value this coroutine yields.
    ///
    /// This associated type corresponds to the `yield` expression and the
    /// values which are allowed to be returned each time a coroutine yields.
    /// For example an iterator-as-a-coroutine would likely have this type as
    /// `T`, the type being iterated over.
    #[lang = "coroutine_yield"]
    type Yield;

    /// The type of value this coroutine returns.
    ///
    /// This corresponds to the type returned from a coroutine either with a
    /// `return` statement or implicitly as the last expression of a coroutine
    /// literal. For example futures would use this as `Result<T, E>` as it
    /// represents a completed future.
    #[lang = "coroutine_return"]
    type Return;

    /// Resumes the execution of this coroutine.
    ///
    /// This function will resume execution of the coroutine or start execution
    /// if it hasn't already. This call will return back into the coroutine's
    /// last suspension point, resuming execution from the latest `yield`. The
    /// coroutine will continue executing until it either yields or returns, at
    /// which point this function will return.
    ///
    /// # Return value
    ///
    /// The `CoroutineState` enum returned from this function indicates what
    /// state the coroutine is in upon returning. If the `Yielded` variant is
    /// returned then the coroutine has reached a suspension point and a value
    /// has been yielded out. Coroutines in this state are available for
    /// resumption at a later point.
    ///
    /// If `Complete` is returned then the coroutine has completely finished
    /// with the value provided. It is invalid for the coroutine to be resumed
    /// again.
    ///
    /// # Panics
    ///
    /// This function may panic if it is called after the `Complete` variant has
    /// been returned previously. While coroutine literals in the language are
    /// guaranteed to panic on resuming after `Complete`, this is not guaranteed
    /// for all implementations of the `Coroutine` trait.
    #[lang = "coroutine_resume"]
    fn resume(self: Pin<&mut Self>, arg: R) -> CoroutineState<Self::Yield, Self::Return>;
}

#[unstable(feature = "coroutine_trait", issue = "43122")]
impl<G: ?Sized + Coroutine<R>, R> Coroutine<R> for Pin<&mut G> {
    type Yield = G::Yield;
    type Return = G::Return;

    fn resume(mut self: Pin<&mut Self>, arg: R) -> CoroutineState<Self::Yield, Self::Return> {
        G::resume((*self).as_mut(), arg)
    }
}

#[unstable(feature = "coroutine_trait", issue = "43122")]
impl<G: ?Sized + Coroutine<R> + Unpin, R> Coroutine<R> for &mut G {
    type Yield = G::Yield;
    type Return = G::Return;

    fn resume(mut self: Pin<&mut Self>, arg: R) -> CoroutineState<Self::Yield, Self::Return> {
        G::resume(Pin::new(&mut *self), arg)
    }
}

}

impl<P:ops::Deref+CoerceUnsized<Q>,Q:ops::Deref>CoerceUnsized<pin::Pin<Q>> for pin::Pin<P>{}
pub fn fixed_factory_unsize<'a>(value:&'a [u8;2])->pin::Pin<&'a [u8]>{pin::Pin::<&'a [u8;2]>::new(value)}

#[lang="drop"]pub trait Drop{fn drop(&mut self);}
unsafe extern "C"{fn record(tag:u8);}
struct Wrap<P>{tag:u8,p:P}
impl<P:CoerceUnsized<Q>,Q>CoerceUnsized<Wrap<Q>>for Wrap<P>{}
struct Outer<P>{tag:u8,p:Wrap<P>}
impl<P:CoerceUnsized<Q>,Q>CoerceUnsized<Outer<Q>>for Outer<P>{}
use marker::PhantomData;
struct Marked<P>{p:P,marker:PhantomData<P>}
impl<P:CoerceUnsized<Q>,Q>CoerceUnsized<Marked<Q>>for Marked<P>{}
struct Tag{value:u8}
impl Drop for Tag{fn drop(&mut self){unsafe{record(self.value)}}}
struct Owned<P>{tag:Tag,p:P}
impl<P:CoerceUnsized<Q>,Q>CoerceUnsized<Owned<Q>>for Owned<P>{}
impl<P>Drop for Owned<P>{fn drop(&mut self){unsafe{record(6)}}}
fn second(value:&[u8])->u8{match value{[_,b]=>*b,_=>0}}
fn first(value:&[u8])->u8{match value{[a,_]=>*a,_=>0}}

#[unsafe(no_mangle)]pub extern "C" fn pin_value()->u8{let a=[23u8,47];let p=fixed_factory_unsize(&a);second(p.pointer)}
#[unsafe(no_mangle)]pub extern "C" fn prefix_value()->u8{let a=[23u8,47];let p:Wrap<&[u8]>=Wrap::<&[u8;2]>{tag:17,p:&a};unsafe{record(p.tag)};first(p.p)}
#[unsafe(no_mangle)]pub extern "C" fn nested_value()->u8{let a=[23u8,47];let p:Outer<&[u8]>=Outer::<&[u8;2]>{tag:19,p:Wrap{tag:17,p:&a}};unsafe{record(p.tag);record(p.p.tag)};second(p.p.p)}
fn make<'a>(a:&'a [u8;2])->Wrap<&'a [u8;2]>{unsafe{record(5)};Wrap{tag:17,p:a}}
#[unsafe(no_mangle)]pub extern "C" fn effect_value()->u8{let a=[23u8,47];let p:Wrap<&[u8]>=make(&a);unsafe{record(p.tag)};first(p.p)}
#[unsafe(no_mangle)]pub extern "C" fn owned_value()->u8{let a=[23u8,47];let p:Owned<&[u8]>=Owned::<&[u8;2]>{tag:Tag{value:17},p:&a};unsafe{record(9)};first(p.p)}
#[unsafe(no_mangle)]pub extern "C" fn phantom_value()->u8{let a=[23u8,47];let p:Marked<&[u8]>=Marked::<&[u8;2]>{p:&a,marker:PhantomData};second(p.p)}

/// Helper function for `drop_in_place`. The compiler replaces this by the actual drop glue.
#[lang = "drop_glue"]
pub(crate) const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{
    // Code here does not matter - this is replaced by the
    // real drop glue by the compiler.
}

