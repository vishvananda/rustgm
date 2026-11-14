#![feature(no_core,lang_items,rustc_attrs,intrinsics,auto_traits)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for u16{fn clone(&self)->Self{*self}}impl Copy for u16{}
impl Clone for u32{fn clone(&self)->Self{*self}}impl Copy for u32{}
impl Clone for u64{fn clone(&self)->Self{*self}}impl Copy for u64{}
impl Clone for u128{fn clone(&self)->Self{*self}}impl Copy for u128{}
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
impl Clone for i8{fn clone(&self)->Self{*self}}impl Copy for i8{}
impl Clone for i16{fn clone(&self)->Self{*self}}impl Copy for i16{}
impl Clone for i32{fn clone(&self)->Self{*self}}impl Copy for i32{}
impl Clone for i64{fn clone(&self)->Self{*self}}impl Copy for i64{}
impl Clone for i128{fn clone(&self)->Self{*self}}impl Copy for i128{}
impl Clone for isize{fn clone(&self)->Self{*self}}impl Copy for isize{}
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_shl<T: Copy, U: Copy>(x: T, y: U) -> T;
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_shr<T: Copy, U: Copy>(x: T, y: U) -> T;
const S0:u64=unsafe{unchecked_shl::<u64,u32>(1,0)};
const S1:u64=unsafe{unchecked_shl::<u64,u32>(1,1)};
const S2:u64=unsafe{unchecked_shl::<u64,u32>(1,2)};
const S3:u64=unsafe{unchecked_shl::<u64,u32>(1,3)};
const S4:u64=unsafe{unchecked_shl::<u64,u32>(1,4)};
const S5:u64=unsafe{unchecked_shl::<u64,u32>(1,5)};
const S6:u64=unsafe{unchecked_shl::<u64,u32>(1,6)};
const S7:u64=unsafe{unchecked_shl::<u64,u32>(1,7)};
const S8:u64=unsafe{unchecked_shl::<u64,u32>(1,8)};
const S9:u64=unsafe{unchecked_shl::<u64,u32>(1,9)};
const S10:u64=unsafe{unchecked_shl::<u64,u32>(1,10)};
const S11:u64=unsafe{unchecked_shl::<u64,u32>(1,11)};
const S12:u64=unsafe{unchecked_shl::<u64,u32>(1,12)};
const S13:u64=unsafe{unchecked_shl::<u64,u32>(1,13)};
const S14:u64=unsafe{unchecked_shl::<u64,u32>(1,14)};
const S15:u64=unsafe{unchecked_shl::<u64,u32>(1,15)};
const S16:u64=unsafe{unchecked_shl::<u64,u32>(1,16)};
const S17:u64=unsafe{unchecked_shl::<u64,u32>(1,17)};
const S18:u64=unsafe{unchecked_shl::<u64,u32>(1,18)};
const S19:u64=unsafe{unchecked_shl::<u64,u32>(1,19)};
const S20:u64=unsafe{unchecked_shl::<u64,u32>(1,20)};
const S21:u64=unsafe{unchecked_shl::<u64,u32>(1,21)};
const S22:u64=unsafe{unchecked_shl::<u64,u32>(1,22)};
const S23:u64=unsafe{unchecked_shl::<u64,u32>(1,23)};
const S24:u64=unsafe{unchecked_shl::<u64,u32>(1,24)};
const S25:u64=unsafe{unchecked_shl::<u64,u32>(1,25)};
const S26:u64=unsafe{unchecked_shl::<u64,u32>(1,26)};
const S27:u64=unsafe{unchecked_shl::<u64,u32>(1,27)};
const S28:u64=unsafe{unchecked_shl::<u64,u32>(1,28)};
const S29:u64=unsafe{unchecked_shl::<u64,u32>(1,29)};
const S30:u64=unsafe{unchecked_shl::<u64,u32>(1,30)};
const S31:u64=unsafe{unchecked_shl::<u64,u32>(1,31)};
