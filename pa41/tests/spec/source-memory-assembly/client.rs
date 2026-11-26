#![feature(no_core,asm_native_fixture)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)] pub unsafe extern "C" fn obs_load(source:usize,dest:usize){unsafe{*(dest as *mut base::__m128i)=base::load::<u32>(source as *const base::__m128i);}}
#[unsafe(no_mangle)] pub unsafe extern "C" fn obs_duplicate(source:usize,dest:usize){unsafe{*(dest as *mut base::__m128i)=base::duplicate::<u32>(*(source as *const base::__m128i));}}
#[unsafe(no_mangle)] pub unsafe extern "C" fn obs_inplace(source:usize,dest:usize){unsafe{*(dest as *mut base::__m128i)=base::inplace::<u32>(*(source as *const base::__m128i));}}
#[unsafe(no_mangle)] pub unsafe extern "C" fn obs_alias(source:usize,dest:usize){unsafe{*(dest as *mut base::__m128i)=base::alias::<u32>(*(source as *const base::__m128i));}}
#[unsafe(no_mangle)] pub unsafe extern "C" fn obs_read(source:usize)->u32{unsafe{base::read::<u32>(source as *const u32)}}
#[unsafe(no_mangle)] pub extern "C" fn obs_ordinary()->u32{base::ordinary::<u32>()}
