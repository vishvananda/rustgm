#![feature(intrinsics,c_variadic)]
use core::ffi::VaList;
#[rustc_intrinsic]pub const unsafe fn va_arg<T>(ap:&mut VaList<'_>)->T;
pub unsafe extern "C" fn probe(_:i32,mut args:...)->bool{unsafe{va_arg::<bool>(&mut args)}}
