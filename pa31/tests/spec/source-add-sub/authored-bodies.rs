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

#[lang = "add"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[rustc_on_unimplemented(
    on(all(Self = "{integer}", Rhs = "{float}"), message = "cannot add a float to an integer",),
    on(all(Self = "{float}", Rhs = "{integer}"), message = "cannot add an integer to a float",),
    message = "cannot add `{Rhs}` to `{Self}`",
    label = "no implementation for `{Self} + {Rhs}`"
)]
#[doc(alias = "+")]
pub const trait Add<Rhs = Self> {
    /// The resulting type after applying the `+` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the `+` operation.
    ///
    /// # Example
    ///
    /// ```
    /// assert_eq!(12 + 1, 13);
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[rustc_diagnostic_item = "add"]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn add(self, rhs: Rhs) -> Self::Output;
}

#[lang = "sub"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "cannot subtract `{Rhs}` from `{Self}`",
    label = "no implementation for `{Self} - {Rhs}`"
)]
#[doc(alias = "-")]
pub const trait Sub<Rhs = Self> {
    /// The resulting type after applying the `-` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the `-` operation.
    ///
    /// # Example
    ///
    /// ```
    /// assert_eq!(12 - 1, 11);
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[rustc_diagnostic_item = "sub"]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn sub(self, rhs: Rhs) -> Self::Output;
}

impl Add for u8{type Output=u8;fn add(self,_:u8)->u8{19}}
impl Sub for u8{type Output=u8;fn sub(self,_:u8)->u8{23}}
impl Add<&u8> for u8{type Output=u8;fn add(self,_:&u8)->u8{29}}impl Sub<&u8> for u8{type Output=u8;fn sub(self,_:&u8)->u8{31}}
#[unsafe(no_mangle)]pub extern "C" fn explicit_add(a:u8,b:u8)->u8{<u8 as Add>::add(a,b)}#[unsafe(no_mangle)]pub extern "C" fn explicit_sub(a:u8,b:u8)->u8{<u8 as Sub>::sub(a,b)}#[unsafe(no_mangle)]pub extern "C" fn scalar_add(a:u8,b:u8)->u8{a+b}#[unsafe(no_mangle)]pub extern "C" fn scalar_sub(a:u8,b:u8)->u8{a-b}#[unsafe(no_mangle)]pub extern "C" fn shared_add(a:u8,b:&u8)->u8{a+b}#[unsafe(no_mangle)]pub extern "C" fn shared_sub(a:u8,b:&u8)->u8{a-b}