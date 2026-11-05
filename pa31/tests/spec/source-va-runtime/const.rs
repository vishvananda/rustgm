#![feature(c_variadic)]
use core::ffi::VaList;
pub const unsafe extern "C" fn probe(_:i32,...)->i32{17}
