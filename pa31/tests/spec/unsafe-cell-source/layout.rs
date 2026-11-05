#![feature(no_core,lang_items,auto_traits,negative_impls,intrinsics,rustc_attrs,freeze_impls,const_trait_impl,const_destruct)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized:PointeeSized {}
#[lang="sized"] pub trait Sized:MetaSized {}
#[lang="legacy_receiver"] pub trait LegacyReceiver:PointeeSized {}
impl<T:PointeeSized> LegacyReceiver for &T {}
impl<T:PointeeSized> LegacyReceiver for &mut T {}
#[lang="clone"] pub trait Clone:Sized { #[lang="clone_fn"] fn clone(&self)->Self; }
#[lang="copy"] pub trait Copy:Clone {}
impl Clone for u32{fn clone(&self)->Self{*self}}impl Copy for u32{}
#[lang="freeze"] pub unsafe auto trait Freeze {}
#[lang="sync"] pub unsafe auto trait Sync {}
#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]pub const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
#[lang="unsafe_cell"] #[repr(transparent)] pub struct Slot<T:?Sized>{value:T}
impl<T> Slot<T>{pub const fn new(value:T)->Self{Self{value}}}
impl<T:?Sized> Slot<T>{pub const fn get(&self)->*mut T{self as *const Self as *mut T}}
impl<T:PointeeSized> !Freeze for Slot<T>{}
impl<T:PointeeSized> !Sync for Slot<T>{}
#[rustc_nounwind] #[rustc_intrinsic] pub const fn size_of<T>()->usize;
pub enum Maybe<T>{None,Some(T)}
#[repr(transparent)]pub struct Plain<T>(T);
#[unsafe(no_mangle)]pub extern "C" fn slot_size()->usize{const{size_of::<Slot<&'static u8>>()}}
#[unsafe(no_mangle)]pub extern "C" fn slot_option_size()->usize{const{size_of::<Maybe<Slot<&'static u8>>>()}}
#[unsafe(no_mangle)]pub extern "C" fn plain_option_size()->usize{const{size_of::<Maybe<Plain<&'static u8>>>()}}
