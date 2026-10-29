#![feature(no_core)]
#![no_core]
#![feature(compat_const)]
extern crate base;
#[unsafe(no_mangle)]pub extern "C" fn const_result()->u8{const{base::gated::const_marked()}}
#[unsafe(no_mangle)]pub extern "C" fn unsafe_result()->u8{unsafe{base::gated::unsafe_marked()}}
#[unsafe(no_mangle)]pub extern "C" fn combined_result()->u8{const{unsafe{base::const_unsafe(23u8)}}}
#[unsafe(no_mangle)]pub extern "C" fn generic_result()->u8{const{base::const_identity(31u8)}}
#[unsafe(no_mangle)]pub extern "C" fn forward_result()->u8{base::const_forward(41u8)}
#[unsafe(no_mangle)]pub extern "C" fn const_forward_result()->u8{const{base::const_forward(57u8)}}
