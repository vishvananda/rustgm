#![feature(no_core,lang_items,intrinsics,rustc_attrs,auto_traits,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
// Option keeps core's declaration (core/src/option.rs): its payload has no niche
// here, so it keeps a tag, which rustc 1.98.0 makes one byte wide.
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="destruct"]pub const trait Destruct:PointeeSized{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for u16{fn clone(&self)->Self{*self}}impl Copy for u16{}
impl Clone for u32{fn clone(&self)->Self{*self}}impl Copy for u32{}
impl Clone for u64{fn clone(&self)->Self{*self}}impl Copy for u64{}
#[rustc_intrinsic]pub const unsafe fn transmute<Src,Dst>(src:Src)->Dst;
#[lang="Option"]pub enum Option<T>{#[lang="None"]None,#[lang="Some"]Some(T)}
#[rustc_intrinsic]pub const unsafe fn write_bytes<T>(dst:*mut T,val:u8,count:usize);
const fn filled(byte:u8)->Option<u8>{let mut v=Option::Some(7u8);unsafe{write_bytes(&mut v as *mut Option<u8>,byte,1usize)};v}
const fn read(v:Option<u8>)->u8{match v{Option::None=>200,Option::Some(1)=>41,Option::Some(_)=>99}}
const ZEROED:u8=read(filled(0));
const ONES:u8=read(filled(1));
#[unsafe(no_mangle)]pub extern "C" fn zeroed()->u8{ZEROED}
#[unsafe(no_mangle)]pub extern "C" fn ones()->u8{ONES}
