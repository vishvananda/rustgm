#![allow(internal_features,unused_features,dead_code,unexpected_cfgs)]
#![feature(no_core,lang_items,abi_unadjusted)]
#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for u64{fn clone(&self)->Self{*self}}impl Copy for u64{}
impl Clone for u128{fn clone(&self)->Self{*self}}impl Copy for u128{}
impl Clone for f32{fn clone(&self)->Self{*self}}impl Copy for f32{}
impl Clone for f64{fn clone(&self)->Self{*self}}impl Copy for f64{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
impl Clone for extern "unadjusted" fn(u64)->u64{fn clone(&self)->Self{*self}}
impl Copy for extern "unadjusted" fn(u64)->u64{}
#[unsafe(no_mangle)]pub extern "unadjusted" fn observed_byte(value:u8)->u8{value}
#[unsafe(no_mangle)]pub extern "unadjusted" fn observed_word(value:u64)->u64{value}
#[unsafe(no_mangle)]pub extern "unadjusted" fn observed_wide(value:u128)->u128{value}
#[unsafe(no_mangle)]pub extern "unadjusted" fn observed_single(value:f32)->f32{value}
#[unsafe(no_mangle)]pub extern "unadjusted" fn observed_double(value:f64)->f64{value}
#[unsafe(no_mangle)]pub extern "unadjusted" fn observed_bit(value:bool)->bool{value}
pub extern "unadjusted" fn identity<T>(value:T)->T{value}
unsafe extern "C"{fn report_values(a:u8,b:u64,c:u128,d:f32,e:f64,f:bool,g:u64,h:u64);}
fn main(){let pointer:extern "unadjusted" fn(u64)->u64=observed_word;unsafe{report_values(observed_byte(255),observed_word(18446744073709551615),observed_wide(170141183460469231731687303715884105729),observed_single(1.5),observed_double(-2.25),observed_bit(true),pointer(37),identity::<u64>(41));}}
