#![feature(no_core,asm_native_fixture)]
#![no_core]
extern crate base;
#[target_feature(enable="avx2")]unsafe fn work_store_i(s:usize,d:usize){unsafe{base::_mm256_stream_si256(d as *mut base::__m256i,*(s as *const base::__m256i));}}
#[unsafe(no_mangle)]pub unsafe extern "C" fn obs_store_i(s:usize,d:usize){unsafe{work_store_i(s,d);}}
#[target_feature(enable="avx2")]unsafe fn work_store_d(s:usize,d:usize){unsafe{base::_mm256_stream_pd(d as *mut f64,*(s as *const base::__m256d));}}
#[unsafe(no_mangle)]pub unsafe extern "C" fn obs_store_d(s:usize,d:usize){unsafe{work_store_d(s,d);}}
#[target_feature(enable="avx2")]unsafe fn work_store_f(s:usize,d:usize){unsafe{base::_mm256_stream_ps(d as *mut f32,*(s as *const base::__m256));}}
#[unsafe(no_mangle)]pub unsafe extern "C" fn obs_store_f(s:usize,d:usize){unsafe{work_store_f(s,d);}}
#[target_feature(enable="avx2")]unsafe fn work_pass(s:usize,d:usize){unsafe{*(d as *mut base::__m256i)=base::pass256::<u32>(*(s as *const base::__m256i));}}
#[unsafe(no_mangle)]pub unsafe extern "C" fn obs_pass(s:usize,d:usize){unsafe{work_pass(s,d);}}
#[target_feature(enable="avx2")]unsafe fn work_narrow128(s:usize,d:usize){unsafe{*(d as *mut base::Narrow128)=base::narrow128_ymm::<u32>(*(s as *const base::Narrow128));}}
#[unsafe(no_mangle)]pub unsafe extern "C" fn obs_narrow128(s:usize,d:usize){unsafe{work_narrow128(s,d);}}
#[target_feature(enable="avx2")]unsafe fn work_load(s:usize,d:usize){unsafe{*(d as *mut base::__m256i)=base::_mm256_stream_load_si256(s as *const base::__m256i);}}
#[unsafe(no_mangle)]pub unsafe extern "C" fn obs_load(s:usize,d:usize){unsafe{work_load(s,d);}}
#[unsafe(no_mangle)]pub extern "C" fn obs_ordinary()->u32{base::ordinary::<u32>()}
