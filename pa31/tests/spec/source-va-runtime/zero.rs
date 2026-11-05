#![feature(c_variadic)]
use core::ffi::VaList;
#[unsafe(no_mangle)]pub unsafe extern "C" fn va_zero(mut args:...)->i32{unsafe{args.next_arg::<i32>()}}
