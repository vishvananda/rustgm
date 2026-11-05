#![feature(c_variadic)]
use core::ffi::VaList;
pub extern "C" fn probe(_:i32,mut args:...)->i32{args.next_arg::<i32>()}
