#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]pub extern "C" fn original()->u8{unsafe{base::forwarded()}}
#[unsafe(no_mangle)]pub extern "C" fn explicit(value:u8)->u8{base::explicit(value)}
#[unsafe(no_mangle)]pub extern "C" fn implicit(value:u8)->u8{unsafe{base::implicit(value)}}
#[unsafe(no_mangle)]pub extern "C" fn explicit_unsafe(value:u8)->u8{unsafe{base::explicit_unsafe(value)}}
#[unsafe(no_mangle)]pub extern "C" fn safe_block(value:u8)->u8{base::safe_in_unsafe(value)}
#[unsafe(no_mangle)]pub extern "C" fn constant()->u8{const{base::const_explicit(37u8)}}
