#![feature(c_variadic)]
use core::ffi::VaList;
pub unsafe extern "C" fn probe(_:i32,mut args:...)->f32{unsafe{args.next_arg::<f32>()}}
