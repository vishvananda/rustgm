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

impl AddAssign<i32> for i32{fn add_assign(&mut self,rhs:i32){*self=rhs as i32;}}
#[unsafe(no_mangle)]pub extern "C" fn primitive_add_assign(left:i32,right:i32)->i32{let mut x=left;x+=right;x}

impl SubAssign<i32> for i32{fn sub_assign(&mut self,rhs:i32){*self=rhs as i32;}}
#[unsafe(no_mangle)]pub extern "C" fn primitive_sub_assign(left:i32,right:i32)->i32{let mut x=left;x-=right;x}

impl MulAssign<i32> for i32{fn mul_assign(&mut self,rhs:i32){*self=rhs as i32;}}
#[unsafe(no_mangle)]pub extern "C" fn primitive_mul_assign(left:i32,right:i32)->i32{let mut x=left;x*=right;x}

impl DivAssign<i32> for i32{fn div_assign(&mut self,rhs:i32){*self=rhs as i32;}}
#[unsafe(no_mangle)]pub extern "C" fn primitive_div_assign(left:i32,right:i32)->i32{let mut x=left;x/=right;x}

impl RemAssign<i32> for i32{fn rem_assign(&mut self,rhs:i32){*self=rhs as i32;}}
#[unsafe(no_mangle)]pub extern "C" fn primitive_rem_assign(left:i32,right:i32)->i32{let mut x=left;x%=right;x}

impl BitXorAssign<i32> for i32{fn bitxor_assign(&mut self,rhs:i32){*self=rhs as i32;}}
#[unsafe(no_mangle)]pub extern "C" fn primitive_bitxor_assign(left:i32,right:i32)->i32{let mut x=left;x^=right;x}
impl BitXorAssign for bool{fn bitxor_assign(&mut self,rhs:bool){*self=rhs;}}
#[unsafe(no_mangle)]pub extern "C" fn boolean_bitxor_assign(left:bool,right:bool)->bool{let mut x=left;x^=right;x}

impl BitAndAssign<i32> for i32{fn bitand_assign(&mut self,rhs:i32){*self=rhs as i32;}}
#[unsafe(no_mangle)]pub extern "C" fn primitive_bitand_assign(left:i32,right:i32)->i32{let mut x=left;x&=right;x}
impl BitAndAssign for bool{fn bitand_assign(&mut self,rhs:bool){*self=rhs;}}
#[unsafe(no_mangle)]pub extern "C" fn boolean_bitand_assign(left:bool,right:bool)->bool{let mut x=left;x&=right;x}

impl BitOrAssign<i32> for i32{fn bitor_assign(&mut self,rhs:i32){*self=rhs as i32;}}
#[unsafe(no_mangle)]pub extern "C" fn primitive_bitor_assign(left:i32,right:i32)->i32{let mut x=left;x|=right;x}
impl BitOrAssign for bool{fn bitor_assign(&mut self,rhs:bool){*self=rhs;}}
#[unsafe(no_mangle)]pub extern "C" fn boolean_bitor_assign(left:bool,right:bool)->bool{let mut x=left;x|=right;x}

impl ShlAssign<u8> for i32{fn shl_assign(&mut self,rhs:u8){*self=rhs as i32;}}
#[unsafe(no_mangle)]pub extern "C" fn primitive_shl_assign(left:i32,right:u8)->i32{let mut x=left;x<<=right;x}

impl ShrAssign<u8> for i32{fn shr_assign(&mut self,rhs:u8){*self=rhs as i32;}}
#[unsafe(no_mangle)]pub extern "C" fn primitive_shr_assign(left:i32,right:u8)->i32{let mut x=left;x>>=right;x}
