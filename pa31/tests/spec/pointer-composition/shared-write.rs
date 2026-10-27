#![feature(ptr_metadata,extern_types)]
#![allow(dead_code)]
use core::{ptr,slice};
static DATA:[u8;3]=[7,8,9];
const BAD:()={let p=ptr::slice_from_raw_parts_mut(&DATA as *const [u8;3] as *const u8 as *mut u8,3);unsafe{(*p)[1]=91;}};
