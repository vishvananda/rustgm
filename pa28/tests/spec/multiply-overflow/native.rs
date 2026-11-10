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
#[unsafe(no_mangle)]pub extern "C" fn result_u8(x:u8,y:u8)->u8{let(value,_flag)=mul_with_overflow(x,y);value}
#[unsafe(no_mangle)]pub extern "C" fn flag_u8(x:u8,y:u8)->u8{let(_value,flag)=mul_with_overflow(x,y);flag as u8}
#[unsafe(no_mangle)]pub extern "C" fn result_u16(x:u16,y:u16)->u16{let(value,_flag)=mul_with_overflow(x,y);value}
#[unsafe(no_mangle)]pub extern "C" fn flag_u16(x:u16,y:u16)->u8{let(_value,flag)=mul_with_overflow(x,y);flag as u8}
#[unsafe(no_mangle)]pub extern "C" fn result_u32(x:u32,y:u32)->u32{let(value,_flag)=mul_with_overflow(x,y);value}
#[unsafe(no_mangle)]pub extern "C" fn flag_u32(x:u32,y:u32)->u8{let(_value,flag)=mul_with_overflow(x,y);flag as u8}
#[unsafe(no_mangle)]pub extern "C" fn result_u64(x:u64,y:u64)->u64{let(value,_flag)=mul_with_overflow(x,y);value}
#[unsafe(no_mangle)]pub extern "C" fn flag_u64(x:u64,y:u64)->u8{let(_value,flag)=mul_with_overflow(x,y);flag as u8}
#[unsafe(no_mangle)]pub extern "C" fn result_u128(x:u128,y:u128)->u128{let(value,_flag)=mul_with_overflow(x,y);value}
#[unsafe(no_mangle)]pub extern "C" fn flag_u128(x:u128,y:u128)->u8{let(_value,flag)=mul_with_overflow(x,y);flag as u8}
#[unsafe(no_mangle)]pub extern "C" fn result_usize(x:usize,y:usize)->usize{let(value,_flag)=mul_with_overflow(x,y);value}
#[unsafe(no_mangle)]pub extern "C" fn flag_usize(x:usize,y:usize)->u8{let(_value,flag)=mul_with_overflow(x,y);flag as u8}
#[unsafe(no_mangle)]pub extern "C" fn result_i8(x:i8,y:i8)->i8{let(value,_flag)=mul_with_overflow(x,y);value}
#[unsafe(no_mangle)]pub extern "C" fn flag_i8(x:i8,y:i8)->u8{let(_value,flag)=mul_with_overflow(x,y);flag as u8}
#[unsafe(no_mangle)]pub extern "C" fn result_i16(x:i16,y:i16)->i16{let(value,_flag)=mul_with_overflow(x,y);value}
#[unsafe(no_mangle)]pub extern "C" fn flag_i16(x:i16,y:i16)->u8{let(_value,flag)=mul_with_overflow(x,y);flag as u8}
#[unsafe(no_mangle)]pub extern "C" fn result_i32(x:i32,y:i32)->i32{let(value,_flag)=mul_with_overflow(x,y);value}
#[unsafe(no_mangle)]pub extern "C" fn flag_i32(x:i32,y:i32)->u8{let(_value,flag)=mul_with_overflow(x,y);flag as u8}
#[unsafe(no_mangle)]pub extern "C" fn result_i64(x:i64,y:i64)->i64{let(value,_flag)=mul_with_overflow(x,y);value}
#[unsafe(no_mangle)]pub extern "C" fn flag_i64(x:i64,y:i64)->u8{let(_value,flag)=mul_with_overflow(x,y);flag as u8}
#[unsafe(no_mangle)]pub extern "C" fn result_i128(x:i128,y:i128)->i128{let(value,_flag)=mul_with_overflow(x,y);value}
#[unsafe(no_mangle)]pub extern "C" fn flag_i128(x:i128,y:i128)->u8{let(_value,flag)=mul_with_overflow(x,y);flag as u8}
#[unsafe(no_mangle)]pub extern "C" fn result_isize(x:isize,y:isize)->isize{let(value,_flag)=mul_with_overflow(x,y);value}
#[unsafe(no_mangle)]pub extern "C" fn flag_isize(x:isize,y:isize)->u8{let(_value,flag)=mul_with_overflow(x,y);flag as u8}
