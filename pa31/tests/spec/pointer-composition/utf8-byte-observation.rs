#![feature(ptr_metadata,extern_types,core_intrinsics)]
#![allow(dead_code,internal_features)]
use core::{ptr,slice};
static DATA:[u8;3]=[7,8,9];
static BAD_BYTES:[u8;1]=[255];const RAW:&str=unsafe{&*ptr::from_raw_parts::<str>(&BAD_BYTES as *const [u8;1] as *const u8,1)};const ANSWER:u8=RAW.as_bytes()[0];#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{ANSWER}
