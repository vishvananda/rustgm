#![feature(ptr_metadata,extern_types)]
#![allow(dead_code)]
use core::{ptr,slice};
static DATA:[u8;3]=[7,8,9];
const ANSWER:usize=ptr::metadata(ptr::slice_from_raw_parts(&DATA as *const [u8;3] as *const u8,99));
#[unsafe(no_mangle)]pub extern "C" fn answer()->usize{ANSWER}
