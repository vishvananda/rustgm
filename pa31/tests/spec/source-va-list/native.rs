#![feature(c_variadic)]
use core::ffi::VaList;
#[unsafe(no_mangle)] pub unsafe extern "C" fn va_read(_:i32,mut args:...) -> i32 {
 let mut copy=args.clone();
 let first=unsafe{args.next_arg::<i32>()};
 let again=unsafe{copy.next_arg::<i32>()};
 let last=unsafe{args.next_arg::<i32>()};
 first.wrapping_add(again).wrapping_add(last)
}
#[unsafe(no_mangle)] pub extern "C" fn va_layout(value:VaList<'_>)->usize {
 let copy=value.clone();
 core::mem::size_of_val(&copy)
}
