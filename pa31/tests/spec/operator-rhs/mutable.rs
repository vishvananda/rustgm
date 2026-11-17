#![feature(auto_traits)]
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

#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="deref"]
pub trait Deref { #[lang="deref_target"] type Target:?Sized; fn deref(&self)->&Self::Target; }
#[lang="deref_mut"]
pub trait DerefMut:Deref { fn deref_mut(&mut self)->&mut Self::Target; }
unsafe extern "C" {fn host_deref();fn host_mut();fn host_add();}
pub struct View(pub u8);
pub struct Owned(pub View);
impl Deref for Owned {type Target=View;fn deref(&self)->&View{unsafe{host_deref();}&self.0}}
pub struct Nest(pub Owned);
impl Deref for Nest {type Target=Owned;fn deref(&self)->&Owned{unsafe{host_deref();}&self.0}}
pub struct Plain(pub View);
pub struct Wrong(pub u8);
impl Deref for Wrong {type Target=u8;fn deref(&self)->&u8{unsafe{host_deref();}&self.0}}
pub struct Mutable(pub View);
impl Deref for Mutable {type Target=View;fn deref(&self)->&View{unsafe{host_deref();}&self.0}}
impl DerefMut for Mutable {fn deref_mut(&mut self)->&mut View{unsafe{host_mut();}&mut self.0}}
pub struct Text(pub u8);
impl<'a> Add<&'a View> for Text {type Output=Text;fn add(self,rhs:&'a View)->Text{unsafe{host_add();}Text(rhs.0)}}
pub struct MutText(pub u8);
impl<'a> Add<&'a mut View> for MutText {type Output=MutText;fn add(self,rhs:&'a mut View)->MutText{unsafe{host_add();}MutText(rhs.0)}}
pub struct RefText;
impl<'a> Add<&'a View> for RefText {type Output=&'a View;fn add(self,rhs:&'a View)->&'a View{unsafe{host_add();}rhs}}
impl<'a> Add<&'a View> for &'a View {type Output=u8;fn add(self,rhs:&'a View)->u8{unsafe{host_add();}rhs.0}}
#[unsafe(no_mangle)]pub extern "C" fn run()->u8{let mut x=Mutable(View(7));(MutText(0)+&mut x).0}
