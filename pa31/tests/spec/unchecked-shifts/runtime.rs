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
#[unsafe(no_mangle)]pub extern "C" fn left_u8(x:u8,n:u32)->u8{unsafe{unchecked_shl(x,n)}}
#[unsafe(no_mangle)]pub extern "C" fn right_i8(x:i8,n:u128)->i8{unsafe{unchecked_shr(x,n)}}
#[unsafe(no_mangle)]pub extern "C" fn left_u128(x:u128,n:i8)->u128{unsafe{unchecked_shl(x,n)}}
#[unsafe(no_mangle)]pub extern "C" fn right_i128(x:i128,n:i8)->i128{unsafe{unchecked_shr(x,n)}}
#[unsafe(no_mangle)]pub extern "C" fn right_isize(x:isize,n:u64)->isize{unsafe{unchecked_shr(x,n)}}
#[unsafe(no_mangle)]pub extern "C" fn left_usize(x:usize,n:u8)->usize{unsafe{unchecked_shl(x,n)}}
