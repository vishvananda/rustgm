#![feature(no_core,lang_items,rustc_attrs,staged_api,auto_traits,const_trait_impl,reborrow)]
#![no_core]
#![unstable(feature="reborrow_fixture",issue="none")]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]pub const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl<T:PointeeSized>Clone for &T{fn clone(&self)->Self{*self}}impl<T:PointeeSized>Copy for &T{}
/// Allows value to be reborrowed as exclusive, creating a copy of the value
/// that disables the source for reads and writes for the lifetime of the copy.
#[lang = "reborrow"]
#[unstable(feature = "reborrow", issue = "145612")]
pub trait Reborrow {
    /* compiler built-in */
}

/// Allows reborrowable value to be reborrowed as shared, creating a copy
/// that disables the source for writes for the lifetime of the copy.
#[lang = "coerce_shared"]
#[unstable(feature = "reborrow", issue = "145612")]
pub trait CoerceShared<Target: Copy>: Reborrow {
    /* compiler built-in */
}

// Authored support uses the actual pinned PhantomData declaration.
#[lang = "phantom_data"]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct PhantomData<T: PointeeSized>;
impl<T:PointeeSized>Clone for PhantomData<T>{fn clone(&self)->Self{Self}}
impl<T:PointeeSized>Copy for PhantomData<T>{}
struct Source<'a>{value:&'a u8}
impl<'a>Reborrow for Source<'a>{}
struct Target<'a>{value:&'static u8,marker:PhantomData<&'a ()>}
impl Clone for Target<'_>{fn clone(&self)->Self{Self{value:self.value,marker:PhantomData}}}
impl Copy for Target<'_>{}
impl<'a>CoerceShared<Target<'a>>for Source<'a>{}
pub fn bad()->&'static u8{let value=7u8;let source=Source{value:&value};let target:Target<'_>=source;target.value}
