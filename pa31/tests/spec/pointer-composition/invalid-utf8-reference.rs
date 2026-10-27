#![feature(ptr_metadata,extern_types)]
#![allow(dead_code)]
use core::{ptr,slice};
static DATA:[u8;3]=[7,8,9];
static BAD_BYTES:[u8;1]=[255];const BAD:&str=unsafe{&*ptr::from_raw_parts::<str>(&BAD_BYTES as *const [u8;1] as *const u8,1)};
