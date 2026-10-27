#![feature(ptr_metadata,extern_types)]
#![allow(dead_code)]
use core::{ptr,slice};
static DATA:[u8;3]=[7,8,9];
const BAD:*const [u8]=ptr::from_raw_parts::<[u8]>(&DATA as *const [u8;3] as *const u8,());
