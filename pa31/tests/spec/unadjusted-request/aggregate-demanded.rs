#![feature(abi_unadjusted)]
#![allow(internal_features,dead_code,improper_ctypes_definitions)]
#[repr(C)]pub struct Triple{pub a:u64,pub b:u64,pub c:u64}
#[unsafe(no_mangle)]pub extern "unadjusted" fn raw(value:Triple)->Triple{value}
#[unsafe(no_mangle)]pub extern "C" fn by_c(value:Triple)->Triple{value}
#[unsafe(no_mangle)]pub extern "C" fn observed()->u64{raw(Triple{a:11,b:13,c:17}).c}
unsafe extern "C"{fn report_value(value:u64);}
fn main(){unsafe{report_value(observed());}}
