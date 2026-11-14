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
impl Clone for f32{fn clone(&self)->Self{*self}}impl Copy for f32{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_add<T:Copy>(x:T,y:T)->T;
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_mul<T:Copy>(x:T,y:T)->T;
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_div<T:Copy>(x:T,y:T)->T;
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_rem<T:Copy>(x:T,y:T)->T;
#[rustc_intrinsic_const_stable_indirect]
#[rustc_nounwind]
#[rustc_intrinsic]
pub const unsafe fn unchecked_sub<T:Copy>(x:T,y:T)->T;
const M0:u64=unsafe{unchecked_mul(1u64,3u64)};
const M1:u64=unsafe{unchecked_mul(2u64,3u64)};
const M2:u64=unsafe{unchecked_mul(3u64,3u64)};
const M3:u64=unsafe{unchecked_mul(4u64,3u64)};
const M4:u64=unsafe{unchecked_mul(5u64,3u64)};
const M5:u64=unsafe{unchecked_mul(6u64,3u64)};
const M6:u64=unsafe{unchecked_mul(7u64,3u64)};
const M7:u64=unsafe{unchecked_mul(8u64,3u64)};
const M8:u64=unsafe{unchecked_mul(9u64,3u64)};
const M9:u64=unsafe{unchecked_mul(10u64,3u64)};
const M10:u64=unsafe{unchecked_mul(11u64,3u64)};
const M11:u64=unsafe{unchecked_mul(12u64,3u64)};
const M12:u64=unsafe{unchecked_mul(13u64,3u64)};
const M13:u64=unsafe{unchecked_mul(14u64,3u64)};
const M14:u64=unsafe{unchecked_mul(15u64,3u64)};
const M15:u64=unsafe{unchecked_mul(16u64,3u64)};
const M16:u64=unsafe{unchecked_mul(17u64,3u64)};
const M17:u64=unsafe{unchecked_mul(18u64,3u64)};
const M18:u64=unsafe{unchecked_mul(19u64,3u64)};
const M19:u64=unsafe{unchecked_mul(20u64,3u64)};
const M20:u64=unsafe{unchecked_mul(21u64,3u64)};
const M21:u64=unsafe{unchecked_mul(22u64,3u64)};
const M22:u64=unsafe{unchecked_mul(23u64,3u64)};
const M23:u64=unsafe{unchecked_mul(24u64,3u64)};
const M24:u64=unsafe{unchecked_mul(25u64,3u64)};
const M25:u64=unsafe{unchecked_mul(26u64,3u64)};
const M26:u64=unsafe{unchecked_mul(27u64,3u64)};
const M27:u64=unsafe{unchecked_mul(28u64,3u64)};
const M28:u64=unsafe{unchecked_mul(29u64,3u64)};
const M29:u64=unsafe{unchecked_mul(30u64,3u64)};
const M30:u64=unsafe{unchecked_mul(31u64,3u64)};
const M31:u64=unsafe{unchecked_mul(32u64,3u64)};
