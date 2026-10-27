#![feature(no_core,lang_items,rustc_attrs,fundamental,staged_api,const_trait_impl,const_destruct)]
#![unstable(feature="source_operator_fixture",issue="none")]
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

// Exact pinned source trait declarations follow. Primitive impl reducers are authored.
#[lang = "shl"]
#[doc(alias = "<<")]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "no implementation for `{Self} << {Rhs}`",
    label = "no implementation for `{Self} << {Rhs}`"
)]
pub const trait Shl<Rhs = Self> {
    /// The resulting type after applying the `<<` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the `<<` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(5u8 << 1, 10);
    /// assert_eq!(1u8 << 1, 2);
    /// ```
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn shl(self, rhs: Rhs) -> Self::Output;
}

#[lang = "bitor"]
#[doc(alias = "|")]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "no implementation for `{Self} | {Rhs}`",
    label = "no implementation for `{Self} | {Rhs}`"
)]
pub const trait BitOr<Rhs = Self> {
    /// The resulting type after applying the `|` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the `|` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(true | false, true);
    /// assert_eq!(false | false, false);
    /// assert_eq!(5u8 | 1u8, 5);
    /// assert_eq!(5u8 | 2u8, 7);
    /// ```
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn bitor(self, rhs: Rhs) -> Self::Output;
}

#[rustc_const_unstable(feature="const_ops",issue="143802")]
const impl Shl<i32> for usize{type Output=usize;fn shl(self,rhs:i32)->usize{self<<rhs}}
#[rustc_const_unstable(feature="const_ops",issue="143802")]
const impl Shl<usize> for usize{type Output=usize;fn shl(self,rhs:usize)->usize{self<<rhs}}
#[rustc_const_unstable(feature="const_ops",issue="143802")]
const impl Shl<u8> for u8{type Output=u8;fn shl(self,rhs:u8)->u8{self<<rhs}}
#[rustc_const_unstable(feature="const_ops",issue="143802")]
const impl BitOr for usize{type Output=usize;fn bitor(self,rhs:usize)->usize{self|rhs}}
#[rustc_const_unstable(feature="const_ops",issue="143802")]
const impl BitOr for bool{type Output=bool;fn bitor(self,rhs:bool)->bool{self|rhs}}
pub const ENCODED:usize=3usize<<1|1;
pub const YES:bool=false|true;
#[unsafe(no_mangle)]pub extern "C" fn encoded(n:usize)->usize{n<<1|1}
