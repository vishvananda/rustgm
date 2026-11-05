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

#[lang = "mul"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "cannot multiply `{Self}` by `{Rhs}`",
    label = "no implementation for `{Self} * {Rhs}`"
)]
#[doc(alias = "*")]
pub const trait Mul<Rhs = Self> {
    /// The resulting type after applying the `*` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the `*` operation.
    ///
    /// # Example
    ///
    /// ```
    /// assert_eq!(12 * 2, 24);
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[rustc_diagnostic_item = "mul"]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn mul(self, rhs: Rhs) -> Self::Output;
}
#[lang = "rem"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "cannot calculate the remainder of `{Self}` divided by `{Rhs}`",
    label = "no implementation for `{Self} % {Rhs}`"
)]
#[doc(alias = "%")]
pub const trait Rem<Rhs = Self> {
    /// The resulting type after applying the `%` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the `%` operation.
    ///
    /// # Example
    ///
    /// ```
    /// assert_eq!(12 % 10, 2);
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[rustc_diagnostic_item = "rem"]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn rem(self, rhs: Rhs) -> Self::Output;
}
#[lang = "bitxor"]
#[doc(alias = "^")]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "no implementation for `{Self} ^ {Rhs}`",
    label = "no implementation for `{Self} ^ {Rhs}`"
)]
pub const trait BitXor<Rhs = Self> {
    /// The resulting type after applying the `^` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the `^` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(true ^ false, true);
    /// assert_eq!(true ^ true, false);
    /// assert_eq!(5u8 ^ 1u8, 4);
    /// assert_eq!(5u8 ^ 2u8, 7);
    /// ```
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn bitxor(self, rhs: Rhs) -> Self::Output;
}
#[lang = "bitand"]
#[doc(alias = "&")]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "no implementation for `{Self} & {Rhs}`",
    label = "no implementation for `{Self} & {Rhs}`"
)]
pub const trait BitAnd<Rhs = Self> {
    /// The resulting type after applying the `&` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the `&` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(true & false, false);
    /// assert_eq!(true & true, true);
    /// assert_eq!(5u8 & 1u8, 1);
    /// assert_eq!(5u8 & 2u8, 0);
    /// ```
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn bitand(self, rhs: Rhs) -> Self::Output;
}
#[lang = "shr"]
#[doc(alias = ">>")]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "no implementation for `{Self} >> {Rhs}`",
    label = "no implementation for `{Self} >> {Rhs}`"
)]
pub const trait Shr<Rhs = Self> {
    /// The resulting type after applying the `>>` operator.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output;

    /// Performs the `>>` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(5u8 >> 1, 2);
    /// assert_eq!(2u8 >> 1, 1);
    /// ```
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn shr(self, rhs: Rhs) -> Self::Output;
}

pub struct Word(i32);pub struct Arg(u8);
impl Mul<Arg> for Word{type Output=i32;fn mul(self,rhs:Arg)->i32{rhs.0 as i32}}
impl Mul for Word{type Output=Word;fn mul(self,rhs:Word)->Word{Word(rhs.0)}}
fn generic_mul<T:Mul<Output=T>>(a:T,b:T)->T{a*b}
#[unsafe(no_mangle)]pub extern "C" fn observe_mul()->i32{Word(10)*Arg(11)}
#[unsafe(no_mangle)]pub extern "C" fn default_mul()->i32{generic_mul(Word(41),Word(42)).0}
impl Rem<Arg> for Word{type Output=i32;fn rem(self,rhs:Arg)->i32{rhs.0 as i32}}
impl Rem for Word{type Output=Word;fn rem(self,rhs:Word)->Word{Word(rhs.0)}}
fn generic_rem<T:Rem<Output=T>>(a:T,b:T)->T{a%b}
#[unsafe(no_mangle)]pub extern "C" fn observe_rem()->i32{Word(12)%Arg(13)}
#[unsafe(no_mangle)]pub extern "C" fn default_rem()->i32{generic_rem(Word(43),Word(44)).0}
impl BitXor<Arg> for Word{type Output=i32;fn bitxor(self,rhs:Arg)->i32{rhs.0 as i32}}
impl BitXor for Word{type Output=Word;fn bitxor(self,rhs:Word)->Word{Word(rhs.0)}}
fn generic_bitxor<T:BitXor<Output=T>>(a:T,b:T)->T{a^b}
#[unsafe(no_mangle)]pub extern "C" fn observe_bitxor()->i32{Word(14)^Arg(15)}
#[unsafe(no_mangle)]pub extern "C" fn default_bitxor()->i32{generic_bitxor(Word(45),Word(46)).0}
impl BitAnd<Arg> for Word{type Output=i32;fn bitand(self,rhs:Arg)->i32{rhs.0 as i32}}
impl BitAnd for Word{type Output=Word;fn bitand(self,rhs:Word)->Word{Word(rhs.0)}}
fn generic_bitand<T:BitAnd<Output=T>>(a:T,b:T)->T{a&b}
#[unsafe(no_mangle)]pub extern "C" fn observe_bitand()->i32{Word(16)&Arg(17)}
#[unsafe(no_mangle)]pub extern "C" fn default_bitand()->i32{generic_bitand(Word(47),Word(48)).0}
impl Shr<Arg> for Word{type Output=i32;fn shr(self,rhs:Arg)->i32{rhs.0 as i32}}
impl Shr for Word{type Output=Word;fn shr(self,rhs:Word)->Word{Word(rhs.0)}}
fn generic_shr<T:Shr<Output=T>>(a:T,b:T)->T{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn observe_shr()->i32{Word(18)>>Arg(19)}
#[unsafe(no_mangle)]pub extern "C" fn default_shr()->i32{generic_shr(Word(49),Word(50)).0}
#[unsafe(no_mangle)]pub extern "C" fn bound_0(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_1(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_2(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_3(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_4(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_5(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_6(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_7(a:Word,b:Arg)->i32{a^b}
