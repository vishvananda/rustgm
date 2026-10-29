#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)] pub extern "C" fn byte()->u8{base::byte()}
#[unsafe(no_mangle)] pub extern "C" fn wide()->u128{base::wide()}
#[unsafe(no_mangle)] pub extern "C" fn signed()->i128{base::signed()}
#[unsafe(no_mangle)] pub extern "C" fn flag()->bool{base::flag()}
#[unsafe(no_mangle)] pub extern "C" fn letter()->char{base::letter()}
#[unsafe(no_mangle)] pub extern "C" fn single()->f32{base::single()}
#[unsafe(no_mangle)] pub extern "C" fn double()->f64{base::double()}
