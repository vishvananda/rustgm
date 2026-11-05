#![feature(c_variadic)]
use core::ffi::VaList;
pub unsafe extern "Rust" fn probe(_:i32,...)->i32{17}
