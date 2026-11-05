#![feature(c_variadic)]
use core::ffi::VaList;
#[unsafe(no_mangle)]pub unsafe extern "C" fn va_int(_:i32,mut args:...)->i32{unsafe{let _=args.next_arg::<i32>();args.next_arg::<i32>()}}
#[unsafe(no_mangle)]pub unsafe extern "C" fn va_copy(_:i32,mut args:...)->i32{let mut other=args.clone();unsafe{let _=args.next_arg::<i32>();other.next_arg::<i32>()}}
#[unsafe(no_mangle)]pub unsafe extern "C" fn va_float(_:i32,mut args:...)->f64{unsafe{let _=args.next_arg::<f64>();args.next_arg::<f64>()}}
#[unsafe(no_mangle)]pub unsafe extern "C" fn va_ptr(_:i32,mut args:...)->*const u8{unsafe{args.next_arg::<*const u8>()}}
#[unsafe(no_mangle)]pub unsafe extern "C" fn va_stack(_:i32,mut args:...)->i32{unsafe{
 let _=args.next_arg::<i32>();let _=args.next_arg::<i32>();let _=args.next_arg::<i32>();let _=args.next_arg::<i32>();
 let _=args.next_arg::<i32>();let _=args.next_arg::<i32>();let _=args.next_arg::<i32>();args.next_arg::<i32>()}}
#[unsafe(no_mangle)]pub unsafe extern "C" fn va_early(_:i32,mut args:...)->i32{return unsafe{args.next_arg::<i32>()};}
#[unsafe(no_mangle)]pub extern "C" fn va_owned(mut args:VaList<'_>)->i32{unsafe{args.next_arg::<i32>()}}
