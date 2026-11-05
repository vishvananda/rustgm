#![feature(c_variadic)]
use core::ffi::VaList;
pub unsafe extern "C" fn probe(_:i32,mut args:...)->i32{let first=&mut args;let second=&mut args;unsafe{first.next_arg::<i32>()}}
