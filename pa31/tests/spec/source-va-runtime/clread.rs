#![feature(const_c_variadic)]
#![feature(c_variadic)]
use core::ffi::VaList;
pub const unsafe fn probe(args:&mut VaList<'_>)->i32{unsafe{args.next_arg::<i32>()}}
