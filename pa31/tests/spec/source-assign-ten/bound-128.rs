#![feature(no_core,lang_items,rustc_attrs,fundamental,staged_api,const_trait_impl,const_destruct)]
#![unstable(feature="source_operator_fixture",issue="none")]
#![feature(auto_traits,const_ops)]
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

#[lang = "freeze"]
#[unstable(feature = "freeze", issue = "121675")]
pub unsafe auto trait Freeze {}
#[lang = "add_assign"]
#[stable(feature = "op_assign_traits", since = "1.8.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "cannot add-assign `{Rhs}` to `{Self}`",
    label = "no implementation for `{Self} += {Rhs}`"
)]
#[doc(alias = "+")]
#[doc(alias = "+=")]
pub const trait AddAssign<Rhs = Self> {
    /// Performs the `+=` operation.
    ///
    /// # Example
    ///
    /// ```
    /// let mut x: u32 = 12;
    /// x += 1;
    /// assert_eq!(x, 13);
    /// ```
    #[stable(feature = "op_assign_traits", since = "1.8.0")]
    fn add_assign(&mut self, rhs: Rhs);
}
#[lang = "sub_assign"]
#[stable(feature = "op_assign_traits", since = "1.8.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "cannot subtract-assign `{Rhs}` from `{Self}`",
    label = "no implementation for `{Self} -= {Rhs}`"
)]
#[doc(alias = "-")]
#[doc(alias = "-=")]
pub const trait SubAssign<Rhs = Self> {
    /// Performs the `-=` operation.
    ///
    /// # Example
    ///
    /// ```
    /// let mut x: u32 = 12;
    /// x -= 1;
    /// assert_eq!(x, 11);
    /// ```
    #[stable(feature = "op_assign_traits", since = "1.8.0")]
    fn sub_assign(&mut self, rhs: Rhs);
}
#[lang = "mul_assign"]
#[stable(feature = "op_assign_traits", since = "1.8.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "cannot multiply-assign `{Self}` by `{Rhs}`",
    label = "no implementation for `{Self} *= {Rhs}`"
)]
#[doc(alias = "*")]
#[doc(alias = "*=")]
pub const trait MulAssign<Rhs = Self> {
    /// Performs the `*=` operation.
    ///
    /// # Example
    ///
    /// ```
    /// let mut x: u32 = 12;
    /// x *= 2;
    /// assert_eq!(x, 24);
    /// ```
    #[stable(feature = "op_assign_traits", since = "1.8.0")]
    fn mul_assign(&mut self, rhs: Rhs);
}
#[lang = "div_assign"]
#[stable(feature = "op_assign_traits", since = "1.8.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "cannot divide-assign `{Self}` by `{Rhs}`",
    label = "no implementation for `{Self} /= {Rhs}`"
)]
#[doc(alias = "/")]
#[doc(alias = "/=")]
pub const trait DivAssign<Rhs = Self> {
    /// Performs the `/=` operation.
    ///
    /// # Example
    ///
    /// ```
    /// let mut x: u32 = 12;
    /// x /= 2;
    /// assert_eq!(x, 6);
    /// ```
    #[stable(feature = "op_assign_traits", since = "1.8.0")]
    fn div_assign(&mut self, rhs: Rhs);
}
#[lang = "rem_assign"]
#[stable(feature = "op_assign_traits", since = "1.8.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "cannot calculate and assign the remainder of `{Self}` divided by `{Rhs}`",
    label = "no implementation for `{Self} %= {Rhs}`"
)]
#[doc(alias = "%")]
#[doc(alias = "%=")]
pub const trait RemAssign<Rhs = Self> {
    /// Performs the `%=` operation.
    ///
    /// # Example
    ///
    /// ```
    /// let mut x: u32 = 12;
    /// x %= 10;
    /// assert_eq!(x, 2);
    /// ```
    #[stable(feature = "op_assign_traits", since = "1.8.0")]
    fn rem_assign(&mut self, rhs: Rhs);
}
#[lang = "bitxor_assign"]
#[doc(alias = "^=")]
#[stable(feature = "op_assign_traits", since = "1.8.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "no implementation for `{Self} ^= {Rhs}`",
    label = "no implementation for `{Self} ^= {Rhs}`"
)]
pub const trait BitXorAssign<Rhs = Self> {
    /// Performs the `^=` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x = true;
    /// x ^= false;
    /// assert_eq!(x, true);
    ///
    /// let mut x = true;
    /// x ^= true;
    /// assert_eq!(x, false);
    ///
    /// let mut x: u8 = 5;
    /// x ^= 1;
    /// assert_eq!(x, 4);
    ///
    /// let mut x: u8 = 5;
    /// x ^= 2;
    /// assert_eq!(x, 7);
    /// ```
    #[stable(feature = "op_assign_traits", since = "1.8.0")]
    fn bitxor_assign(&mut self, rhs: Rhs);
}
#[lang = "bitand_assign"]
#[doc(alias = "&=")]
#[stable(feature = "op_assign_traits", since = "1.8.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "no implementation for `{Self} &= {Rhs}`",
    label = "no implementation for `{Self} &= {Rhs}`"
)]
pub const trait BitAndAssign<Rhs = Self> {
    /// Performs the `&=` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x = true;
    /// x &= false;
    /// assert_eq!(x, false);
    ///
    /// let mut x = true;
    /// x &= true;
    /// assert_eq!(x, true);
    ///
    /// let mut x: u8 = 5;
    /// x &= 1;
    /// assert_eq!(x, 1);
    ///
    /// let mut x: u8 = 5;
    /// x &= 2;
    /// assert_eq!(x, 0);
    /// ```
    #[stable(feature = "op_assign_traits", since = "1.8.0")]
    fn bitand_assign(&mut self, rhs: Rhs);
}
#[lang = "bitor_assign"]
#[doc(alias = "|=")]
#[stable(feature = "op_assign_traits", since = "1.8.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "no implementation for `{Self} |= {Rhs}`",
    label = "no implementation for `{Self} |= {Rhs}`"
)]
pub const trait BitOrAssign<Rhs = Self> {
    /// Performs the `|=` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x = true;
    /// x |= false;
    /// assert_eq!(x, true);
    ///
    /// let mut x = false;
    /// x |= false;
    /// assert_eq!(x, false);
    ///
    /// let mut x: u8 = 5;
    /// x |= 1;
    /// assert_eq!(x, 5);
    ///
    /// let mut x: u8 = 5;
    /// x |= 2;
    /// assert_eq!(x, 7);
    /// ```
    #[stable(feature = "op_assign_traits", since = "1.8.0")]
    fn bitor_assign(&mut self, rhs: Rhs);
}
#[lang = "shl_assign"]
#[doc(alias = "<<=")]
#[stable(feature = "op_assign_traits", since = "1.8.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "no implementation for `{Self} <<= {Rhs}`",
    label = "no implementation for `{Self} <<= {Rhs}`"
)]
pub const trait ShlAssign<Rhs = Self> {
    /// Performs the `<<=` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x: u8 = 5;
    /// x <<= 1;
    /// assert_eq!(x, 10);
    ///
    /// let mut x: u8 = 1;
    /// x <<= 1;
    /// assert_eq!(x, 2);
    /// ```
    #[stable(feature = "op_assign_traits", since = "1.8.0")]
    fn shl_assign(&mut self, rhs: Rhs);
}
#[lang = "shr_assign"]
#[doc(alias = ">>=")]
#[stable(feature = "op_assign_traits", since = "1.8.0")]
#[rustc_const_unstable(feature = "const_ops", issue = "143802")]
#[diagnostic::on_unimplemented(
    message = "no implementation for `{Self} >>= {Rhs}`",
    label = "no implementation for `{Self} >>= {Rhs}`"
)]
pub const trait ShrAssign<Rhs = Self> {
    /// Performs the `>>=` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x: u8 = 5;
    /// x >>= 1;
    /// assert_eq!(x, 2);
    ///
    /// let mut x: u8 = 2;
    /// x >>= 1;
    /// assert_eq!(x, 1);
    /// ```
    #[stable(feature = "op_assign_traits", since = "1.8.0")]
    fn shr_assign(&mut self, rhs: Rhs);
}

pub struct Word(i32);pub struct Arg(i32);
impl AddAssign<Arg> for Word{fn add_assign(&mut self,rhs:Arg){self.0=rhs.0;}}
impl AddAssign for Word{fn add_assign(&mut self,rhs:Word){self.0=rhs.0;}}
fn generic_add_assign<T:AddAssign>(x:&mut T,rhs:T){*x+=rhs;}
#[unsafe(no_mangle)]pub extern "C" fn custom_add_assign(left:i32,right:i32)->i32{let mut x=Word(left);x+=Arg(right);x.0}
#[unsafe(no_mangle)]pub extern "C" fn default_add_assign(left:i32,right:i32)->i32{let mut x=Word(left);generic_add_assign(&mut x,Word(right));x.0}
impl SubAssign<Arg> for Word{fn sub_assign(&mut self,rhs:Arg){self.0=rhs.0;}}
impl SubAssign for Word{fn sub_assign(&mut self,rhs:Word){self.0=rhs.0;}}
fn generic_sub_assign<T:SubAssign>(x:&mut T,rhs:T){*x-=rhs;}
#[unsafe(no_mangle)]pub extern "C" fn custom_sub_assign(left:i32,right:i32)->i32{let mut x=Word(left);x-=Arg(right);x.0}
#[unsafe(no_mangle)]pub extern "C" fn default_sub_assign(left:i32,right:i32)->i32{let mut x=Word(left);generic_sub_assign(&mut x,Word(right));x.0}
impl MulAssign<Arg> for Word{fn mul_assign(&mut self,rhs:Arg){self.0=rhs.0;}}
impl MulAssign for Word{fn mul_assign(&mut self,rhs:Word){self.0=rhs.0;}}
fn generic_mul_assign<T:MulAssign>(x:&mut T,rhs:T){*x*=rhs;}
#[unsafe(no_mangle)]pub extern "C" fn custom_mul_assign(left:i32,right:i32)->i32{let mut x=Word(left);x*=Arg(right);x.0}
#[unsafe(no_mangle)]pub extern "C" fn default_mul_assign(left:i32,right:i32)->i32{let mut x=Word(left);generic_mul_assign(&mut x,Word(right));x.0}
impl DivAssign<Arg> for Word{fn div_assign(&mut self,rhs:Arg){self.0=rhs.0;}}
impl DivAssign for Word{fn div_assign(&mut self,rhs:Word){self.0=rhs.0;}}
fn generic_div_assign<T:DivAssign>(x:&mut T,rhs:T){*x/=rhs;}
#[unsafe(no_mangle)]pub extern "C" fn custom_div_assign(left:i32,right:i32)->i32{let mut x=Word(left);x/=Arg(right);x.0}
#[unsafe(no_mangle)]pub extern "C" fn default_div_assign(left:i32,right:i32)->i32{let mut x=Word(left);generic_div_assign(&mut x,Word(right));x.0}
impl RemAssign<Arg> for Word{fn rem_assign(&mut self,rhs:Arg){self.0=rhs.0;}}
impl RemAssign for Word{fn rem_assign(&mut self,rhs:Word){self.0=rhs.0;}}
fn generic_rem_assign<T:RemAssign>(x:&mut T,rhs:T){*x%=rhs;}
#[unsafe(no_mangle)]pub extern "C" fn custom_rem_assign(left:i32,right:i32)->i32{let mut x=Word(left);x%=Arg(right);x.0}
#[unsafe(no_mangle)]pub extern "C" fn default_rem_assign(left:i32,right:i32)->i32{let mut x=Word(left);generic_rem_assign(&mut x,Word(right));x.0}
impl BitXorAssign<Arg> for Word{fn bitxor_assign(&mut self,rhs:Arg){self.0=rhs.0;}}
impl BitXorAssign for Word{fn bitxor_assign(&mut self,rhs:Word){self.0=rhs.0;}}
fn generic_bitxor_assign<T:BitXorAssign>(x:&mut T,rhs:T){*x^=rhs;}
#[unsafe(no_mangle)]pub extern "C" fn custom_bitxor_assign(left:i32,right:i32)->i32{let mut x=Word(left);x^=Arg(right);x.0}
#[unsafe(no_mangle)]pub extern "C" fn default_bitxor_assign(left:i32,right:i32)->i32{let mut x=Word(left);generic_bitxor_assign(&mut x,Word(right));x.0}
impl BitAndAssign<Arg> for Word{fn bitand_assign(&mut self,rhs:Arg){self.0=rhs.0;}}
impl BitAndAssign for Word{fn bitand_assign(&mut self,rhs:Word){self.0=rhs.0;}}
fn generic_bitand_assign<T:BitAndAssign>(x:&mut T,rhs:T){*x&=rhs;}
#[unsafe(no_mangle)]pub extern "C" fn custom_bitand_assign(left:i32,right:i32)->i32{let mut x=Word(left);x&=Arg(right);x.0}
#[unsafe(no_mangle)]pub extern "C" fn default_bitand_assign(left:i32,right:i32)->i32{let mut x=Word(left);generic_bitand_assign(&mut x,Word(right));x.0}
impl BitOrAssign<Arg> for Word{fn bitor_assign(&mut self,rhs:Arg){self.0=rhs.0;}}
impl BitOrAssign for Word{fn bitor_assign(&mut self,rhs:Word){self.0=rhs.0;}}
fn generic_bitor_assign<T:BitOrAssign>(x:&mut T,rhs:T){*x|=rhs;}
#[unsafe(no_mangle)]pub extern "C" fn custom_bitor_assign(left:i32,right:i32)->i32{let mut x=Word(left);x|=Arg(right);x.0}
#[unsafe(no_mangle)]pub extern "C" fn default_bitor_assign(left:i32,right:i32)->i32{let mut x=Word(left);generic_bitor_assign(&mut x,Word(right));x.0}
impl ShlAssign<Arg> for Word{fn shl_assign(&mut self,rhs:Arg){self.0=rhs.0;}}
impl ShlAssign for Word{fn shl_assign(&mut self,rhs:Word){self.0=rhs.0;}}
fn generic_shl_assign<T:ShlAssign>(x:&mut T,rhs:T){*x<<=rhs;}
#[unsafe(no_mangle)]pub extern "C" fn custom_shl_assign(left:i32,right:i32)->i32{let mut x=Word(left);x<<=Arg(right);x.0}
#[unsafe(no_mangle)]pub extern "C" fn default_shl_assign(left:i32,right:i32)->i32{let mut x=Word(left);generic_shl_assign(&mut x,Word(right));x.0}
impl ShrAssign<Arg> for Word{fn shr_assign(&mut self,rhs:Arg){self.0=rhs.0;}}
impl ShrAssign for Word{fn shr_assign(&mut self,rhs:Word){self.0=rhs.0;}}
fn generic_shr_assign<T:ShrAssign>(x:&mut T,rhs:T){*x>>=rhs;}
#[unsafe(no_mangle)]pub extern "C" fn custom_shr_assign(left:i32,right:i32)->i32{let mut x=Word(left);x>>=Arg(right);x.0}
#[unsafe(no_mangle)]pub extern "C" fn default_shr_assign(left:i32,right:i32)->i32{let mut x=Word(left);generic_shr_assign(&mut x,Word(right));x.0}
pub fn bound_0(x:&mut Word,rhs:Arg){*x+=rhs;}
pub fn bound_1(x:&mut Word,rhs:Arg){*x-=rhs;}
pub fn bound_2(x:&mut Word,rhs:Arg){*x*=rhs;}
pub fn bound_3(x:&mut Word,rhs:Arg){*x/=rhs;}
pub fn bound_4(x:&mut Word,rhs:Arg){*x%=rhs;}
pub fn bound_5(x:&mut Word,rhs:Arg){*x^=rhs;}
pub fn bound_6(x:&mut Word,rhs:Arg){*x&=rhs;}
pub fn bound_7(x:&mut Word,rhs:Arg){*x|=rhs;}
pub fn bound_8(x:&mut Word,rhs:Arg){*x<<=rhs;}
pub fn bound_9(x:&mut Word,rhs:Arg){*x>>=rhs;}
pub fn bound_10(x:&mut Word,rhs:Arg){*x+=rhs;}
pub fn bound_11(x:&mut Word,rhs:Arg){*x-=rhs;}
pub fn bound_12(x:&mut Word,rhs:Arg){*x*=rhs;}
pub fn bound_13(x:&mut Word,rhs:Arg){*x/=rhs;}
pub fn bound_14(x:&mut Word,rhs:Arg){*x%=rhs;}
pub fn bound_15(x:&mut Word,rhs:Arg){*x^=rhs;}
pub fn bound_16(x:&mut Word,rhs:Arg){*x&=rhs;}
pub fn bound_17(x:&mut Word,rhs:Arg){*x|=rhs;}
pub fn bound_18(x:&mut Word,rhs:Arg){*x<<=rhs;}
pub fn bound_19(x:&mut Word,rhs:Arg){*x>>=rhs;}
pub fn bound_20(x:&mut Word,rhs:Arg){*x+=rhs;}
pub fn bound_21(x:&mut Word,rhs:Arg){*x-=rhs;}
pub fn bound_22(x:&mut Word,rhs:Arg){*x*=rhs;}
pub fn bound_23(x:&mut Word,rhs:Arg){*x/=rhs;}
pub fn bound_24(x:&mut Word,rhs:Arg){*x%=rhs;}
pub fn bound_25(x:&mut Word,rhs:Arg){*x^=rhs;}
pub fn bound_26(x:&mut Word,rhs:Arg){*x&=rhs;}
pub fn bound_27(x:&mut Word,rhs:Arg){*x|=rhs;}
pub fn bound_28(x:&mut Word,rhs:Arg){*x<<=rhs;}
pub fn bound_29(x:&mut Word,rhs:Arg){*x>>=rhs;}
pub fn bound_30(x:&mut Word,rhs:Arg){*x+=rhs;}
pub fn bound_31(x:&mut Word,rhs:Arg){*x-=rhs;}
pub fn bound_32(x:&mut Word,rhs:Arg){*x*=rhs;}
pub fn bound_33(x:&mut Word,rhs:Arg){*x/=rhs;}
pub fn bound_34(x:&mut Word,rhs:Arg){*x%=rhs;}
pub fn bound_35(x:&mut Word,rhs:Arg){*x^=rhs;}
pub fn bound_36(x:&mut Word,rhs:Arg){*x&=rhs;}
pub fn bound_37(x:&mut Word,rhs:Arg){*x|=rhs;}
pub fn bound_38(x:&mut Word,rhs:Arg){*x<<=rhs;}
pub fn bound_39(x:&mut Word,rhs:Arg){*x>>=rhs;}
pub fn bound_40(x:&mut Word,rhs:Arg){*x+=rhs;}
pub fn bound_41(x:&mut Word,rhs:Arg){*x-=rhs;}
pub fn bound_42(x:&mut Word,rhs:Arg){*x*=rhs;}
pub fn bound_43(x:&mut Word,rhs:Arg){*x/=rhs;}
pub fn bound_44(x:&mut Word,rhs:Arg){*x%=rhs;}
pub fn bound_45(x:&mut Word,rhs:Arg){*x^=rhs;}
pub fn bound_46(x:&mut Word,rhs:Arg){*x&=rhs;}
pub fn bound_47(x:&mut Word,rhs:Arg){*x|=rhs;}
pub fn bound_48(x:&mut Word,rhs:Arg){*x<<=rhs;}
pub fn bound_49(x:&mut Word,rhs:Arg){*x>>=rhs;}
pub fn bound_50(x:&mut Word,rhs:Arg){*x+=rhs;}
pub fn bound_51(x:&mut Word,rhs:Arg){*x-=rhs;}
pub fn bound_52(x:&mut Word,rhs:Arg){*x*=rhs;}
pub fn bound_53(x:&mut Word,rhs:Arg){*x/=rhs;}
pub fn bound_54(x:&mut Word,rhs:Arg){*x%=rhs;}
pub fn bound_55(x:&mut Word,rhs:Arg){*x^=rhs;}
pub fn bound_56(x:&mut Word,rhs:Arg){*x&=rhs;}
pub fn bound_57(x:&mut Word,rhs:Arg){*x|=rhs;}
pub fn bound_58(x:&mut Word,rhs:Arg){*x<<=rhs;}
pub fn bound_59(x:&mut Word,rhs:Arg){*x>>=rhs;}
pub fn bound_60(x:&mut Word,rhs:Arg){*x+=rhs;}
pub fn bound_61(x:&mut Word,rhs:Arg){*x-=rhs;}
pub fn bound_62(x:&mut Word,rhs:Arg){*x*=rhs;}
pub fn bound_63(x:&mut Word,rhs:Arg){*x/=rhs;}
pub fn bound_64(x:&mut Word,rhs:Arg){*x%=rhs;}
pub fn bound_65(x:&mut Word,rhs:Arg){*x^=rhs;}
pub fn bound_66(x:&mut Word,rhs:Arg){*x&=rhs;}
pub fn bound_67(x:&mut Word,rhs:Arg){*x|=rhs;}
pub fn bound_68(x:&mut Word,rhs:Arg){*x<<=rhs;}
pub fn bound_69(x:&mut Word,rhs:Arg){*x>>=rhs;}
pub fn bound_70(x:&mut Word,rhs:Arg){*x+=rhs;}
pub fn bound_71(x:&mut Word,rhs:Arg){*x-=rhs;}
pub fn bound_72(x:&mut Word,rhs:Arg){*x*=rhs;}
pub fn bound_73(x:&mut Word,rhs:Arg){*x/=rhs;}
pub fn bound_74(x:&mut Word,rhs:Arg){*x%=rhs;}
pub fn bound_75(x:&mut Word,rhs:Arg){*x^=rhs;}
pub fn bound_76(x:&mut Word,rhs:Arg){*x&=rhs;}
pub fn bound_77(x:&mut Word,rhs:Arg){*x|=rhs;}
pub fn bound_78(x:&mut Word,rhs:Arg){*x<<=rhs;}
pub fn bound_79(x:&mut Word,rhs:Arg){*x>>=rhs;}
pub fn bound_80(x:&mut Word,rhs:Arg){*x+=rhs;}
pub fn bound_81(x:&mut Word,rhs:Arg){*x-=rhs;}
pub fn bound_82(x:&mut Word,rhs:Arg){*x*=rhs;}
pub fn bound_83(x:&mut Word,rhs:Arg){*x/=rhs;}
pub fn bound_84(x:&mut Word,rhs:Arg){*x%=rhs;}
pub fn bound_85(x:&mut Word,rhs:Arg){*x^=rhs;}
pub fn bound_86(x:&mut Word,rhs:Arg){*x&=rhs;}
pub fn bound_87(x:&mut Word,rhs:Arg){*x|=rhs;}
pub fn bound_88(x:&mut Word,rhs:Arg){*x<<=rhs;}
pub fn bound_89(x:&mut Word,rhs:Arg){*x>>=rhs;}
pub fn bound_90(x:&mut Word,rhs:Arg){*x+=rhs;}
pub fn bound_91(x:&mut Word,rhs:Arg){*x-=rhs;}
pub fn bound_92(x:&mut Word,rhs:Arg){*x*=rhs;}
pub fn bound_93(x:&mut Word,rhs:Arg){*x/=rhs;}
pub fn bound_94(x:&mut Word,rhs:Arg){*x%=rhs;}
pub fn bound_95(x:&mut Word,rhs:Arg){*x^=rhs;}
pub fn bound_96(x:&mut Word,rhs:Arg){*x&=rhs;}
pub fn bound_97(x:&mut Word,rhs:Arg){*x|=rhs;}
pub fn bound_98(x:&mut Word,rhs:Arg){*x<<=rhs;}
pub fn bound_99(x:&mut Word,rhs:Arg){*x>>=rhs;}
pub fn bound_100(x:&mut Word,rhs:Arg){*x+=rhs;}
pub fn bound_101(x:&mut Word,rhs:Arg){*x-=rhs;}
pub fn bound_102(x:&mut Word,rhs:Arg){*x*=rhs;}
pub fn bound_103(x:&mut Word,rhs:Arg){*x/=rhs;}
pub fn bound_104(x:&mut Word,rhs:Arg){*x%=rhs;}
pub fn bound_105(x:&mut Word,rhs:Arg){*x^=rhs;}
pub fn bound_106(x:&mut Word,rhs:Arg){*x&=rhs;}
pub fn bound_107(x:&mut Word,rhs:Arg){*x|=rhs;}
pub fn bound_108(x:&mut Word,rhs:Arg){*x<<=rhs;}
pub fn bound_109(x:&mut Word,rhs:Arg){*x>>=rhs;}
pub fn bound_110(x:&mut Word,rhs:Arg){*x+=rhs;}
pub fn bound_111(x:&mut Word,rhs:Arg){*x-=rhs;}
pub fn bound_112(x:&mut Word,rhs:Arg){*x*=rhs;}
pub fn bound_113(x:&mut Word,rhs:Arg){*x/=rhs;}
pub fn bound_114(x:&mut Word,rhs:Arg){*x%=rhs;}
pub fn bound_115(x:&mut Word,rhs:Arg){*x^=rhs;}
pub fn bound_116(x:&mut Word,rhs:Arg){*x&=rhs;}
pub fn bound_117(x:&mut Word,rhs:Arg){*x|=rhs;}
pub fn bound_118(x:&mut Word,rhs:Arg){*x<<=rhs;}
pub fn bound_119(x:&mut Word,rhs:Arg){*x>>=rhs;}
pub fn bound_120(x:&mut Word,rhs:Arg){*x+=rhs;}
pub fn bound_121(x:&mut Word,rhs:Arg){*x-=rhs;}
pub fn bound_122(x:&mut Word,rhs:Arg){*x*=rhs;}
pub fn bound_123(x:&mut Word,rhs:Arg){*x/=rhs;}
pub fn bound_124(x:&mut Word,rhs:Arg){*x%=rhs;}
pub fn bound_125(x:&mut Word,rhs:Arg){*x^=rhs;}
pub fn bound_126(x:&mut Word,rhs:Arg){*x&=rhs;}
pub fn bound_127(x:&mut Word,rhs:Arg){*x|=rhs;}
