#![feature(ptr_metadata,extern_types)]
#![allow(dead_code)]
use core::{ptr,slice};
static DATA:[u8;3]=[7,8,9];
const fn meta()->(){let _=unsafe{*(0usize as *const u8)};}const BAD:*const u8=ptr::from_raw_parts::<u8>(1usize as *const u8,meta());
