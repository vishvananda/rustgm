#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#[unsafe(no_mangle)]pub extern "C" fn observed_docs()->u8{7}#[unsafe(no_mangle)]pub extern "C" fn observed_pointer()->u8{let value:u8=9;let p=&value as *const u8;let address=p as usize;let q=address as *const u8;unsafe{*q}}
