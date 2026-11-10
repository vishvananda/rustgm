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
#[rustc_intrinsic_const_stable_indirect]#[rustc_nounwind]#[rustc_intrinsic]pub const fn mul_with_overflow<T:Copy>(x:T,y:T)->(T,bool);
pub fn force(){
let _check:[u8;{let (_result,flag)=mul_with_overflow(255u8 as u8,2u8);flag as usize}]=[0;1];
let _check:[u8;{let (_result,flag)=mul_with_overflow(65535u16 as u16,2u16);flag as usize}]=[0;1];
let _check:[u8;{let (_result,flag)=mul_with_overflow(4294967295u32 as u32,2u32);flag as usize}]=[0;1];
let _check:[u8;{let (_result,flag)=mul_with_overflow(18446744073709551615u64 as u64,2u64);flag as usize}]=[0;1];
let _check:[u8;{let (_result,flag)=mul_with_overflow(340282366920938463463374607431768211455u128 as u128,2u128);flag as usize}]=[0;1];
let _check:[u8;{let (_result,flag)=mul_with_overflow(18446744073709551615u64 as usize,2usize);flag as usize}]=[0;1];
let _check:[u8;{let (_result,flag)=mul_with_overflow(128u8 as i8,255u8 as i8);flag as usize}]=[0;1];
let _check:[u8;{let (_result,flag)=mul_with_overflow(32768u16 as i16,65535u16 as i16);flag as usize}]=[0;1];
let _check:[u8;{let (_result,flag)=mul_with_overflow(2147483648u32 as i32,4294967295u32 as i32);flag as usize}]=[0;1];
let _check:[u8;{let (_result,flag)=mul_with_overflow(9223372036854775808u64 as i64,18446744073709551615u64 as i64);flag as usize}]=[0;1];
let _check:[u8;{let (_result,flag)=mul_with_overflow(170141183460469231731687303715884105728u128 as i128,340282366920938463463374607431768211455u128 as i128);flag as usize}]=[0;1];
let _check:[u8;{let (_result,flag)=mul_with_overflow(9223372036854775808u64 as isize,18446744073709551615u64 as isize);flag as usize}]=[0;1];
}
