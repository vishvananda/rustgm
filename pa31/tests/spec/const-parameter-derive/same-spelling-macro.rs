#![feature(core_intrinsics,coverage_attribute,multiple_supertrait_upcastable,unsize,coerce_unsized,dispatch_from_dyn)]
#![feature(decl_macro,allow_internal_unstable,min_adt_const_params,const_param_ty_trait,structural_match,auto_traits,const_precise_live_drops,source_operator_fixture,no_core,lang_items,rustc_attrs,fundamental,staged_api,const_trait_impl,const_destruct,intrinsics)]
#![unstable(feature="source_operator_fixture",issue="none")]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
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
 use super::DiscriminantKind;
 #[unstable(feature="core_intrinsics",issue="none")]#[rustc_intrinsic_const_stable_indirect]#[rustc_nounwind]#[rustc_intrinsic]
 pub const fn discriminant_value<T>(v:&T)-><T as DiscriminantKind>::Discriminant;
}
extern crate self as core;
pub mod cmp {pub use crate::PartialEq;use crate::PointeeSized;
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
impl Formatter<'_>{
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn write_str(&mut self, data: &str) -> Result {
        self.buf.write_str(data)
    }
}
}

use crate::fmt::Debug;use crate::marker::ConstParamTy;use crate::cmp::Eq;
macro_rules! ConstParamTy {($t:item)=>{$t}}
/// A type for alignment options for SIMD masked load/store intrinsics.
#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum SimdAlign {
    // These values must match the compiler's `SimdAlign` defined in
    // `rustc_middle/src/ty/consts/int.rs`!
    /// No alignment requirements on the pointer
    Unaligned = 0,
    /// The pointer must be aligned to the element type of the SIMD vector
    Element = 1,
    /// The pointer must be aligned to the SIMD vector type
    Vector = 2,
}


fn require_actual_marker<T:marker::ConstParamTy_>(value:T)->T {value}
#[unsafe(no_mangle)] pub extern "C" fn observed()->u8 {
 let value=require_actual_marker(SimdAlign::Element);
 if value==SimdAlign::Element {7} else {0}
}
