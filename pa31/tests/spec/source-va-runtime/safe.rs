#![feature(c_variadic)]
use core::ffi::VaList;
pub extern "C" fn probe(_:i32,...)->i32{17}
