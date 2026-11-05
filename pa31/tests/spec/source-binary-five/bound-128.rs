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
#[unsafe(no_mangle)]pub extern "C" fn bound_8(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_9(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_10(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_11(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_12(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_13(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_14(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_15(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_16(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_17(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_18(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_19(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_20(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_21(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_22(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_23(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_24(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_25(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_26(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_27(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_28(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_29(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_30(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_31(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_32(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_33(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_34(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_35(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_36(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_37(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_38(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_39(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_40(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_41(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_42(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_43(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_44(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_45(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_46(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_47(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_48(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_49(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_50(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_51(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_52(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_53(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_54(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_55(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_56(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_57(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_58(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_59(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_60(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_61(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_62(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_63(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_64(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_65(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_66(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_67(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_68(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_69(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_70(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_71(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_72(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_73(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_74(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_75(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_76(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_77(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_78(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_79(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_80(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_81(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_82(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_83(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_84(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_85(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_86(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_87(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_88(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_89(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_90(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_91(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_92(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_93(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_94(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_95(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_96(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_97(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_98(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_99(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_100(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_101(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_102(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_103(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_104(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_105(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_106(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_107(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_108(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_109(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_110(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_111(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_112(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_113(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_114(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_115(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_116(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_117(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_118(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_119(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_120(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_121(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_122(a:Word,b:Arg)->i32{a^b}
#[unsafe(no_mangle)]pub extern "C" fn bound_123(a:Word,b:Arg)->i32{a&b}
#[unsafe(no_mangle)]pub extern "C" fn bound_124(a:Word,b:Arg)->i32{a>>b}
#[unsafe(no_mangle)]pub extern "C" fn bound_125(a:Word,b:Arg)->i32{a*b}
#[unsafe(no_mangle)]pub extern "C" fn bound_126(a:Word,b:Arg)->i32{a%b}
#[unsafe(no_mangle)]pub extern "C" fn bound_127(a:Word,b:Arg)->i32{a^b}
