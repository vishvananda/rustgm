#![feature(cfg_target_has_reliable_f16_f128)]
#![allow(internal_features,unused_macros,dead_code,unexpected_cfgs)]
#![feature(no_core,lang_items,rustc_attrs,allow_internal_unstable)]
#![no_core]
#[rustc_builtin_macro]macro_rules!cfg{($($t:tt)*)=>{}}
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[unsafe(no_mangle)]pub extern "C" fn observed_0()->u8{if cfg!(target_has_reliable_f16){1}else{0}}
#[unsafe(no_mangle)]pub extern "C" fn observed_1()->u8{if cfg!(target_has_reliable_f16_math){1}else{0}}
#[unsafe(no_mangle)]pub extern "C" fn observed_2()->u8{if cfg!(target_has_reliable_f128){1}else{0}}
#[unsafe(no_mangle)]pub extern "C" fn observed_3()->u8{if cfg!(target_has_reliable_f128_math){1}else{0}}

unsafe extern "C" {fn report_values(a:u8,b:u8,c:u8,d:u8);}
fn main(){unsafe{report_values(observed_0(),observed_1(),observed_2(),observed_3());}}
