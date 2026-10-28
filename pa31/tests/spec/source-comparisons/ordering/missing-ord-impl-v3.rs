#![feature(auto_traits,const_precise_live_drops,source_operator_fixture,no_core,lang_items,rustc_attrs,fundamental,staged_api,const_trait_impl,const_destruct,intrinsics,auto_traits,unboxed_closures,core_intrinsics)]
#![unstable(feature="source_operator_fixture",issue="none")]
#![no_core]
#![rustc_coherence_is_core]
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

partial_eq_impl!{usize u16 u8 i8 bool}


// Authored minimal support declarations. Bodies below are unchanged source chunks.
#[lang="tuple_trait"]#[rustc_deny_explicit_impl]pub trait Tuple{}
#[rustc_const_unstable(feature="const_trait_impl",issue="143874")]#[lang="fn_once"]pub const trait FnOnce<Args:Tuple>{#[lang="fn_once_output"]type Output;extern "rust-call" fn call_once(self,args:Args)->Self::Output;}
#[rustc_const_unstable(feature="const_trait_impl",issue="143874")]#[lang="fn_mut"]pub const trait FnMut<Args:Tuple>:FnOnce<Args>{extern "rust-call" fn call_mut(&mut self,args:Args)->Self::Output;}
#[rustc_const_unstable(feature="const_trait_impl",issue="143874")]#[lang="fn"]pub const trait Fn<Args:Tuple>:[const]FnMut<Args>{extern "rust-call" fn call(&self,args:Args)->Self::Output;}
#[lang="Option"]pub enum Option<T>{#[lang="None"]None,#[lang="Some"]Some(T)}
use Option::{None,Some};
pub enum ControlFlow<B,C=()>{Continue(C),Break(B)}
use ControlFlow::{Continue,Break};
#[lang="discriminant_kind"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub trait DiscriminantKind{#[lang="discriminant_type"]type Discriminant:Sized;}
pub mod intrinsics{
 use super::{Copy,DiscriminantKind,Ordering};
 #[unstable(feature="core_intrinsics",issue="none")]#[rustc_intrinsic_const_stable_indirect]#[rustc_nounwind]#[rustc_intrinsic]
 pub const fn discriminant_value<T>(v:&T)-><T as DiscriminantKind>::Discriminant;
#[unstable(feature="core_intrinsics",issue="none")]#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const fn three_way_compare<T: Copy>(lhs: T, rhss: T) -> crate::cmp::Ordering;
}
extern crate self as core;
impl u16{#[unstable(feature="source_operator_fixture",issue="none")]pub const MAX:u16=65535;}
pub mod cmp{pub use crate::{PartialEq,PartialOrd,Ordering};}

#[lang = "neg"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[doc(alias = "-")]
pub const trait Neg {
    /// The resulting type after applying the `-` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the unary `-` operation.
    ///
    /// # Example
    ///
    /// ```
    /// let x: i32 = 12;
    /// assert_eq!(-x, -12);
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[rustc_diagnostic_item = "neg"]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn neg(self) -> Self::Output;
}

macro_rules! neg_impl {
    ($($t:ty)*) => ($(
        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")]
        const impl Neg for $t {
            type Output = $t;

            #[inline]
            #[track_caller]
            #[rustc_inherit_overflow_checks]
            fn neg(self) -> $t { -self }
        }

        forward_ref_unop! { impl Neg, neg for $t,
        #[stable(feature = "rust1", since = "1.0.0")]
        #[rustc_const_unstable(feature = "const_ops", issue = "143802")] }
    )*)
}


neg_impl!{i8}

#[lang = "Ordering"]
#[repr(i8)]
pub enum Ordering {
    /// An ordering where a compared value is less than another.
    #[stable(feature = "rust1", since = "1.0.0")]
    Less = -1,
    /// An ordering where a compared value is equal to another.
    #[stable(feature = "rust1", since = "1.0.0")]
    Equal = 0,
    /// An ordering where a compared value is greater than another.
    #[stable(feature = "rust1", since = "1.0.0")]
    Greater = 1,
}

use Ordering::{Less,Equal,Greater};
impl Ordering {
    #[inline]
    const fn as_raw(self) -> i8 {
        // FIXME(const-hack): just use `PartialOrd` against `Equal` once that's const
        crate::intrinsics::discriminant_value(&self)
    }

    /// Returns `true` if the ordering is the `Equal` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(Ordering::Less.is_eq(), false);
    /// assert_eq!(Ordering::Equal.is_eq(), true);
    /// assert_eq!(Ordering::Greater.is_eq(), false);
    /// ```
    #[inline]
    #[must_use]
    #[rustc_const_stable(feature = "ordering_helpers", since = "1.53.0")]
    #[stable(feature = "ordering_helpers", since = "1.53.0")]
    pub const fn is_eq(self) -> bool {
        // All the `is_*` methods are implemented as comparisons against zero
        // to follow how clang's libcxx implements their equivalents in
        // <https://github.com/llvm/llvm-project/blob/60486292b79885b7800b082754153202bef5b1f0/libcxx/include/__compare/is_eq.h#L23-L28>

        self.as_raw() == 0
    }

    /// Returns `true` if the ordering is not the `Equal` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(Ordering::Less.is_ne(), true);
    /// assert_eq!(Ordering::Equal.is_ne(), false);
    /// assert_eq!(Ordering::Greater.is_ne(), true);
    /// ```
    #[inline]
    #[must_use]
    #[rustc_const_stable(feature = "ordering_helpers", since = "1.53.0")]
    #[stable(feature = "ordering_helpers", since = "1.53.0")]
    pub const fn is_ne(self) -> bool {
        self.as_raw() != 0
    }

    /// Returns `true` if the ordering is the `Less` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(Ordering::Less.is_lt(), true);
    /// assert_eq!(Ordering::Equal.is_lt(), false);
    /// assert_eq!(Ordering::Greater.is_lt(), false);
    /// ```
    #[inline]
    #[must_use]
    #[rustc_const_stable(feature = "ordering_helpers", since = "1.53.0")]
    #[stable(feature = "ordering_helpers", since = "1.53.0")]
    pub const fn is_lt(self) -> bool {
        self.as_raw() < 0
    }

    /// Returns `true` if the ordering is the `Greater` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(Ordering::Less.is_gt(), false);
    /// assert_eq!(Ordering::Equal.is_gt(), false);
    /// assert_eq!(Ordering::Greater.is_gt(), true);
    /// ```
    #[inline]
    #[must_use]
    #[rustc_const_stable(feature = "ordering_helpers", since = "1.53.0")]
    #[stable(feature = "ordering_helpers", since = "1.53.0")]
    pub const fn is_gt(self) -> bool {
        self.as_raw() > 0
    }

    /// Returns `true` if the ordering is either the `Less` or `Equal` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(Ordering::Less.is_le(), true);
    /// assert_eq!(Ordering::Equal.is_le(), true);
    /// assert_eq!(Ordering::Greater.is_le(), false);
    /// ```
    #[inline]
    #[must_use]
    #[rustc_const_stable(feature = "ordering_helpers", since = "1.53.0")]
    #[stable(feature = "ordering_helpers", since = "1.53.0")]
    pub const fn is_le(self) -> bool {
        self.as_raw() <= 0
    }

    /// Returns `true` if the ordering is either the `Greater` or `Equal` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(Ordering::Less.is_ge(), false);
    /// assert_eq!(Ordering::Equal.is_ge(), true);
    /// assert_eq!(Ordering::Greater.is_ge(), true);
    /// ```
    #[inline]
    #[must_use]
    #[rustc_const_stable(feature = "ordering_helpers", since = "1.53.0")]
    #[stable(feature = "ordering_helpers", since = "1.53.0")]
    pub const fn is_ge(self) -> bool {
        self.as_raw() >= 0
    }

}

impl Clone for Ordering{fn clone(&self)->Self{*self}}impl Copy for Ordering{}
impl<T> Option<T>{
    #[must_use]
    #[inline]
    #[stable(feature = "is_some_and", since = "1.70.0")]
    #[rustc_const_unstable(feature = "const_option_ops", issue = "143956")]
    pub const fn is_some_and(self, f: impl [const] FnOnce(T) -> bool + [const] Destruct) -> bool {
        match self {
            None => false,
            Some(x) => f(x),
        }
    }

}
#[lang = "partial_ord"]
#[stable(feature = "rust1", since = "1.0.0")]
#[doc(alias = ">")]
#[doc(alias = "<")]
#[doc(alias = "<=")]
#[doc(alias = ">=")]
#[diagnostic::on_unimplemented(
    message = "can't compare `{Self}` with `{Rhs}`",
    label = "no implementation for `{Self} < {Rhs}` and `{Self} > {Rhs}`"
)]
#[rustc_diagnostic_item = "PartialOrd"]
#[allow(multiple_supertrait_upcastable)] // FIXME(sized_hierarchy): remove this
#[rustc_const_unstable(feature = "const_cmp", issue = "143800")]
pub const trait PartialOrd<Rhs: PointeeSized = Self>:
    [const] PartialEq<Rhs> + PointeeSized
{
    /// This method returns an ordering between `self` and `other` values if one exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    ///
    /// let result = 1.0.partial_cmp(&2.0);
    /// assert_eq!(result, Some(Ordering::Less));
    ///
    /// let result = 1.0.partial_cmp(&1.0);
    /// assert_eq!(result, Some(Ordering::Equal));
    ///
    /// let result = 2.0.partial_cmp(&1.0);
    /// assert_eq!(result, Some(Ordering::Greater));
    /// ```
    ///
    /// When comparison is impossible:
    ///
    /// ```
    /// let result = f64::NAN.partial_cmp(&1.0);
    /// assert_eq!(result, None);
    /// ```
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_diagnostic_item = "cmp_partialord_cmp"]
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering>;

    /// Tests less than (for `self` and `other`) and is used by the `<` operator.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(1.0 < 1.0, false);
    /// assert_eq!(1.0 < 2.0, true);
    /// assert_eq!(2.0 < 1.0, false);
    /// ```
    #[inline]
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_diagnostic_item = "cmp_partialord_lt"]
    fn lt(&self, other: &Rhs) -> bool {
        self.partial_cmp(other).is_some_and(Ordering::is_lt)
    }

    /// Tests less than or equal to (for `self` and `other`) and is used by the
    /// `<=` operator.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(1.0 <= 1.0, true);
    /// assert_eq!(1.0 <= 2.0, true);
    /// assert_eq!(2.0 <= 1.0, false);
    /// ```
    #[inline]
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_diagnostic_item = "cmp_partialord_le"]
    fn le(&self, other: &Rhs) -> bool {
        self.partial_cmp(other).is_some_and(Ordering::is_le)
    }

    /// Tests greater than (for `self` and `other`) and is used by the `>`
    /// operator.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(1.0 > 1.0, false);
    /// assert_eq!(1.0 > 2.0, false);
    /// assert_eq!(2.0 > 1.0, true);
    /// ```
    #[inline]
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_diagnostic_item = "cmp_partialord_gt"]
    fn gt(&self, other: &Rhs) -> bool {
        self.partial_cmp(other).is_some_and(Ordering::is_gt)
    }

    /// Tests greater than or equal to (for `self` and `other`) and is used by
    /// the `>=` operator.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(1.0 >= 1.0, true);
    /// assert_eq!(1.0 >= 2.0, false);
    /// assert_eq!(2.0 >= 1.0, true);
    /// ```
    #[inline]
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_diagnostic_item = "cmp_partialord_ge"]
    fn ge(&self, other: &Rhs) -> bool {
        self.partial_cmp(other).is_some_and(Ordering::is_ge)
    }

    /// If `self == other`, returns `ControlFlow::Continue(())`.
    /// Otherwise, returns `ControlFlow::Break(self < other)`.
    ///
    /// This is useful for chaining together calls when implementing a lexical
    /// `PartialOrd::lt`, as it allows types (like primitives) which can cheaply
    /// check `==` and `<` separately to do rather than needing to calculate
    /// (then optimize out) the three-way `Ordering` result.
    #[inline]
    // Added to improve the behaviour of tuples; not necessarily stabilization-track.
    #[unstable(feature = "partial_ord_chaining_methods", issue = "none")]
    #[doc(hidden)]
    fn __chaining_lt(&self, other: &Rhs) -> ControlFlow<bool> {
        default_chaining_impl(self, other, Ordering::is_lt)
    }

    /// Same as `__chaining_lt`, but for `<=` instead of `<`.
    #[inline]
    #[unstable(feature = "partial_ord_chaining_methods", issue = "none")]
    #[doc(hidden)]
    fn __chaining_le(&self, other: &Rhs) -> ControlFlow<bool> {
        default_chaining_impl(self, other, Ordering::is_le)
    }

    /// Same as `__chaining_lt`, but for `>` instead of `<`.
    #[inline]
    #[unstable(feature = "partial_ord_chaining_methods", issue = "none")]
    #[doc(hidden)]
    fn __chaining_gt(&self, other: &Rhs) -> ControlFlow<bool> {
        default_chaining_impl(self, other, Ordering::is_gt)
    }

    /// Same as `__chaining_lt`, but for `>=` instead of `<`.
    #[inline]
    #[unstable(feature = "partial_ord_chaining_methods", issue = "none")]
    #[doc(hidden)]
    fn __chaining_ge(&self, other: &Rhs) -> ControlFlow<bool> {
        default_chaining_impl(self, other, Ordering::is_ge)
    }
}

#[rustc_const_unstable(feature = "const_cmp", issue = "143800")]
const fn default_chaining_impl<T, U>(
    lhs: &T,
    rhs: &U,
    p: impl [const] FnOnce(Ordering) -> bool + [const] Destruct,
) -> ControlFlow<bool>
where
    T: [const] PartialOrd<U> + PointeeSized,
    U: PointeeSized,
{
    // It's important that this only call `partial_cmp` once, not call `eq` then
    // one of the relational operators.  We don't want to `bcmp`-then-`memcp` a
    // `String`, for example, or similarly for other data structures (#108157).
    match <T as PartialOrd<U>>::partial_cmp(lhs, rhs) {
        Some(Equal) => ControlFlow::Continue(()),
        Some(c) => ControlFlow::Break(p(c)),
        None => ControlFlow::Break(false),
    }
}

    #[rustfmt::skip]
    macro_rules! partial_ord_methods_primitive_impl {
        () => {
            #[inline(always)]
            fn lt(&self, other: &Self) -> bool { *self <  *other }
            #[inline(always)]
            fn le(&self, other: &Self) -> bool { *self <= *other }
            #[inline(always)]
            fn gt(&self, other: &Self) -> bool { *self >  *other }
            #[inline(always)]
            fn ge(&self, other: &Self) -> bool { *self >= *other }

            // These implementations are the same for `Ord` or `PartialOrd` types
            // because if either is NAN the `==` test will fail so we end up in
            // the `Break` case and the comparison will correctly return `false`.

            #[inline]
            fn __chaining_lt(&self, other: &Self) -> ControlFlow<bool> {
                let (lhs, rhs) = (*self, *other);
                if lhs == rhs { Continue(()) } else { Break(lhs < rhs) }
            }
            #[inline]
            fn __chaining_le(&self, other: &Self) -> ControlFlow<bool> {
                let (lhs, rhs) = (*self, *other);
                if lhs == rhs { Continue(()) } else { Break(lhs <= rhs) }
            }
            #[inline]
            fn __chaining_gt(&self, other: &Self) -> ControlFlow<bool> {
                let (lhs, rhs) = (*self, *other);
                if lhs == rhs { Continue(()) } else { Break(lhs > rhs) }
            }
            #[inline]
            fn __chaining_ge(&self, other: &Self) -> ControlFlow<bool> {
                let (lhs, rhs) = (*self, *other);
                if lhs == rhs { Continue(()) } else { Break(lhs >= rhs) }
            }
        };
    }

macro_rules! integer_partial_ord_probe{($($t:ty)*)=>($(
            #[stable(feature = "rust1", since = "1.0.0")]
            #[rustc_const_unstable(feature = "const_cmp", issue = "143800")]
            const impl PartialOrd for $t {
                #[inline]
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    Some(crate::intrinsics::three_way_compare(*self, *other))
                }

                partial_ord_methods_primitive_impl!();
            }


)*)}
integer_partial_ord_probe!{u16 u8 i8}fn bad(a:usize,b:usize)->bool{a>b}
