#![feature(c_variadic)]
use core::ffi::VaList;
pub unsafe extern "C" fn probe(_:i32,args:...)->VaList<'static>{args}
