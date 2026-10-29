#![feature(no_core,lang_items,rustc_attrs,const_trait_impl,auto_traits,negative_impls,staged_api)]
#![no_core]
#![unstable(feature="unsafe_unpin_fixture",issue="none")]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="clone"]#[rustc_const_unstable(feature="const_clone",issue="142757")]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
// Authored structural payload reducer; original unsafe_pinned lang role remains Required.
struct UnsafePinned<T:PointeeSized>{value:T}
#[lang = "unsafe_unpin"]
#[unstable(feature = "unsafe_unpin", issue = "125735")]
pub unsafe auto trait UnsafeUnpin {}

#[unstable(feature = "unsafe_unpin", issue = "125735")]
impl<T: PointeeSized> !UnsafeUnpin for UnsafePinned<T> {}
#[lang="phantom_data"]pub struct PhantomData<T:PointeeSized>;
#[unstable(feature="unsafe_unpin",issue="125735")]
unsafe impl<T:PointeeSized> UnsafeUnpin for PhantomData<T>{}
#[unstable(feature="unsafe_unpin",issue="125735")]
impl<T:PointeeSized> UnsafeUnpin for *const T{}
#[unstable(feature="unsafe_unpin",issue="125735")]
unsafe impl<T:PointeeSized> UnsafeUnpin for *mut T{}
#[unstable(feature="unsafe_unpin",issue="125735")]
unsafe impl<T:PointeeSized> UnsafeUnpin for &T{}
#[unstable(feature="unsafe_unpin",issue="125735")]
unsafe impl<T:PointeeSized> UnsafeUnpin for &mut T{}
fn need<T:UnsafeUnpin>()->u8{7}
struct W<T>(T);
fn forwarded<T:UnsafeUnpin>()->u8{need::<W<T>>()}
#[unsafe(no_mangle)]pub extern "C" fn observed()->u8{
let _=need::<*const UnsafePinned<u8>>();let _=need::<*mut UnsafePinned<u8>>();
let _=need::<&UnsafePinned<u8>>();let _=need::<&mut UnsafePinned<u8>>();
let _=need::<PhantomData<UnsafePinned<u8>>>();let _=forwarded::<u8>();need::<W<(u8,[u8;2])>>()}
