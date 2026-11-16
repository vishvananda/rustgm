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
const fn read8(v:Option<u8>)->u8{match v{Option::None=>200,Option::Some(x)=>x}}
const fn read32(v:Option<u32>)->u32{match v{Option::None=>200,Option::Some(x)=>x}}
const BAD:u8=read8(unsafe{transmute::<u16,Option<u8>>(0x0002)});
#[unsafe(no_mangle)]pub extern "C" fn bad()->u8{BAD}
