#![feature(no_core,asm_native_fixture)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]pub extern "C" fn obs_eax(leaf:u32,sub_leaf:u32)->u32 {base::query::<u32>(leaf,sub_leaf).eax}
#[unsafe(no_mangle)]pub extern "C" fn obs_ebx(leaf:u32,sub_leaf:u32)->u32 {base::query::<u32>(leaf,sub_leaf).ebx}
#[unsafe(no_mangle)]pub extern "C" fn obs_ecx(leaf:u32,sub_leaf:u32)->u32 {base::query::<u32>(leaf,sub_leaf).ecx}
#[unsafe(no_mangle)]pub extern "C" fn obs_edx(leaf:u32,sub_leaf:u32)->u32 {base::query::<u32>(leaf,sub_leaf).edx}
#[unsafe(no_mangle)]pub extern "C" fn obs_signed(value:i32)->i32 {base::increment::<u32>(value)}
#[unsafe(no_mangle)]pub extern "C" fn obs_ordinary()->u32 {base::ordinary::<u32>(123)}
#[unsafe(no_mangle)]pub extern "C" fn obs_overlap()->u32 {base::late_split::<u32>(31)}
#[unsafe(no_mangle)]pub extern "C" fn obs_reverse()->u32 {base::late_reverse::<u32>(41)}
#[unsafe(no_mangle)]pub unsafe extern "C" fn obs_stream(address:usize,value:i32){unsafe{base::_mm_stream_si32(address as *mut i32,value);}}
#[unsafe(no_mangle)]pub extern "C" fn obs_narrow(value:u64)->u64{base::narrow::<u32>(value)}
