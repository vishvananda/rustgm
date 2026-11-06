#![feature(no_core,lang_items,rustc_attrs,staged_api,auto_traits,const_trait_impl,negative_impls)]
#![no_core]
#![unstable(feature="custom_auto_fixture",issue="none")]
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
pub mod marker{#[lang="phantom_data"]pub struct PhantomData<T:super::PointeeSized>;}
use marker::PhantomData;
impl<T:PointeeSized>Clone for PhantomData<T>{fn clone(&self)->Self{PhantomData}}
impl<T:PointeeSized>Copy for PhantomData<T>{}
pub mod cell{use crate::Sized;
#[lang = "unsafe_cell"]
#[stable(feature = "rust1", since = "1.0.0")]
#[repr(transparent)]
#[rustc_pub_transparent]
pub struct UnsafeCell<T: ?Sized> {
    value: T,
}
}
use cell::UnsafeCell;
#[stable(feature = "catch_unwind", since = "1.9.0")]
#[rustc_diagnostic_item = "unwind_safe_trait"]
#[diagnostic::on_unimplemented(
    message = "the type `{Self}` may not be safely transferred across an unwind boundary",
    label = "`{Self}` may not be safely transferred across an unwind boundary"
)]
pub auto trait UnwindSafe {}

/// A marker trait representing types where a shared reference is considered
/// unwind safe.
///
/// This trait is namely not implemented by [`UnsafeCell`], the root of all
/// interior mutability.
///
/// This is a "helper marker trait" used to provide impl blocks for the
/// [`UnwindSafe`] trait, for more information see that documentation.
#[stable(feature = "catch_unwind", since = "1.9.0")]
#[rustc_diagnostic_item = "ref_unwind_safe_trait"]
#[diagnostic::on_unimplemented(
    message = "the type `{Self}` may contain interior mutability and a reference may not be safely \
               transferable across a catch_unwind boundary",
    label = "`{Self}` may contain interior mutability and a reference may not be safely \
             transferable across a catch_unwind boundary"
)]
pub auto trait RefUnwindSafe {}

#[stable(feature = "catch_unwind", since = "1.9.0")]
pub struct AssertUnwindSafe<T>(#[stable(feature = "catch_unwind", since = "1.9.0")] pub T);

#[stable(feature = "catch_unwind", since = "1.9.0")]
impl<T: PointeeSized> !UnwindSafe for &mut T {}
#[stable(feature = "catch_unwind", since = "1.9.0")]
impl<T: RefUnwindSafe + PointeeSized> UnwindSafe for &T {}
#[stable(feature = "catch_unwind", since = "1.9.0")]
impl<T: RefUnwindSafe + PointeeSized> UnwindSafe for *const T {}
#[stable(feature = "catch_unwind", since = "1.9.0")]
impl<T: RefUnwindSafe + PointeeSized> UnwindSafe for *mut T {}

#[stable(feature = "catch_unwind", since = "1.9.0")]
impl<T> UnwindSafe for AssertUnwindSafe<T> {}

// Pretty simple implementations for the `RefUnwindSafe` marker trait,
// basically just saying that `UnsafeCell` is the
// only thing which doesn't implement it (which then transitively applies to
// everything else).
#[stable(feature = "catch_unwind", since = "1.9.0")]
impl<T: ?Sized> !RefUnwindSafe for UnsafeCell<T> {}
#[stable(feature = "catch_unwind", since = "1.9.0")]
impl<T> RefUnwindSafe for AssertUnwindSafe<T> {}

