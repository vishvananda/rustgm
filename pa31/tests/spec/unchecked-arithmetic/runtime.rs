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
#[unsafe(no_mangle)]pub extern "C" fn add_u8(x:u8,y:u8)->u8{unsafe{unchecked_add(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn add_u16(x:u16,y:u16)->u16{unsafe{unchecked_add(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn add_u32(x:u32,y:u32)->u32{unsafe{unchecked_add(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn add_u64(x:u64,y:u64)->u64{unsafe{unchecked_add(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn add_u128(x:u128,y:u128)->u128{unsafe{unchecked_add(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn add_usize(x:usize,y:usize)->usize{unsafe{unchecked_add(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn add_i8(x:i8,y:i8)->i8{unsafe{unchecked_add(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn add_i16(x:i16,y:i16)->i16{unsafe{unchecked_add(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn add_i32(x:i32,y:i32)->i32{unsafe{unchecked_add(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn add_i64(x:i64,y:i64)->i64{unsafe{unchecked_add(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn add_i128(x:i128,y:i128)->i128{unsafe{unchecked_add(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn add_isize(x:isize,y:isize)->isize{unsafe{unchecked_add(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn mul_u8(x:u8,y:u8)->u8{unsafe{unchecked_mul(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn mul_u16(x:u16,y:u16)->u16{unsafe{unchecked_mul(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn mul_u32(x:u32,y:u32)->u32{unsafe{unchecked_mul(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn mul_u64(x:u64,y:u64)->u64{unsafe{unchecked_mul(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn mul_u128(x:u128,y:u128)->u128{unsafe{unchecked_mul(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn mul_usize(x:usize,y:usize)->usize{unsafe{unchecked_mul(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn mul_i8(x:i8,y:i8)->i8{unsafe{unchecked_mul(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn mul_i16(x:i16,y:i16)->i16{unsafe{unchecked_mul(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn mul_i32(x:i32,y:i32)->i32{unsafe{unchecked_mul(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn mul_i64(x:i64,y:i64)->i64{unsafe{unchecked_mul(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn mul_i128(x:i128,y:i128)->i128{unsafe{unchecked_mul(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn mul_isize(x:isize,y:isize)->isize{unsafe{unchecked_mul(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn div_u8(x:u8,y:u8)->u8{unsafe{unchecked_div(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn div_u16(x:u16,y:u16)->u16{unsafe{unchecked_div(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn div_u32(x:u32,y:u32)->u32{unsafe{unchecked_div(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn div_u64(x:u64,y:u64)->u64{unsafe{unchecked_div(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn div_u128(x:u128,y:u128)->u128{unsafe{unchecked_div(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn div_usize(x:usize,y:usize)->usize{unsafe{unchecked_div(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn div_i8(x:i8,y:i8)->i8{unsafe{unchecked_div(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn div_i16(x:i16,y:i16)->i16{unsafe{unchecked_div(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn div_i32(x:i32,y:i32)->i32{unsafe{unchecked_div(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn div_i64(x:i64,y:i64)->i64{unsafe{unchecked_div(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn div_i128(x:i128,y:i128)->i128{unsafe{unchecked_div(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn div_isize(x:isize,y:isize)->isize{unsafe{unchecked_div(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn rem_u8(x:u8,y:u8)->u8{unsafe{unchecked_rem(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn rem_u16(x:u16,y:u16)->u16{unsafe{unchecked_rem(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn rem_u32(x:u32,y:u32)->u32{unsafe{unchecked_rem(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn rem_u64(x:u64,y:u64)->u64{unsafe{unchecked_rem(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn rem_u128(x:u128,y:u128)->u128{unsafe{unchecked_rem(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn rem_usize(x:usize,y:usize)->usize{unsafe{unchecked_rem(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn rem_i8(x:i8,y:i8)->i8{unsafe{unchecked_rem(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn rem_i16(x:i16,y:i16)->i16{unsafe{unchecked_rem(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn rem_i32(x:i32,y:i32)->i32{unsafe{unchecked_rem(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn rem_i64(x:i64,y:i64)->i64{unsafe{unchecked_rem(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn rem_i128(x:i128,y:i128)->i128{unsafe{unchecked_rem(x,y)}}
#[unsafe(no_mangle)]pub extern "C" fn rem_isize(x:isize,y:isize)->isize{unsafe{unchecked_rem(x,y)}}
