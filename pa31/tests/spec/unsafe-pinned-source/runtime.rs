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
#[lang="unpin"]pub auto trait Unpin{}
#[lang="unsafe_unpin"]pub unsafe auto trait UnsafeUnpin{}
#[lang="unsafe_pinned"]#[repr(transparent)]pub struct Anchor<T:?Sized>{value:Slot<T>}
impl<T> Anchor<T>{pub const fn new(value:T)->Self{Self{value:Slot::new(value)}}}
impl<T:?Sized> Anchor<T>{pub const fn get(&self)->*mut T{self.value.get()}}
impl<T:?Sized> !Unpin for Anchor<T>{}
impl<T:PointeeSized> !UnsafeUnpin for Anchor<T>{}
unsafe impl<T:?Sized+Sync> Sync for Anchor<T>{}
pub struct Global(Anchor<u32>);unsafe impl Sync for Global{}
static STORE:Global=Global(Anchor::new(4u32));static ORDINARY:u32=7;
#[unsafe(no_mangle)]pub extern "C" fn local_write(value:u32)->u32{let cell=Anchor::new(3u32);let alias=&cell;unsafe{*cell.get()=value;*alias.get()}}
#[unsafe(no_mangle)]pub extern "C" fn global_write(value:u32)->u32{unsafe{*STORE.0.get()=value;*STORE.0.get()}}
fn alias_write(left:&mut Anchor<u32>,right:&mut Anchor<u32>)->u32{unsafe{*left.get()=21;*right.get()=23;*left.get()}}
#[unsafe(no_mangle)]pub extern "C" fn aliased_write()->u32{let mut cell=Anchor::new(3u32);let raw=&mut cell as *mut Anchor<u32>;unsafe{alias_write(&mut *raw,&mut *raw)}}
#[unsafe(no_mangle)]pub extern "C" fn plain_address()->*const u32{&ORDINARY}
