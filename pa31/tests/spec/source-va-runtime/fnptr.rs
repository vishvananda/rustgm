#![feature(c_variadic)]
use core::ffi::VaList;
pub type Function=unsafe extern "C" fn(i32,...)->i32;
