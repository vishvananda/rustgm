#![feature(ptr_metadata,extern_types)]
#![allow(dead_code)]
use core::{ptr,slice};
static DATA:[u8;3]=[7,8,9];
const ANSWER:usize={let p={let x=7u8;&x as *const u8};ptr::metadata(ptr::slice_from_raw_parts(p,3))};
#[unsafe(no_mangle)]pub extern "C" fn answer()->usize{ANSWER}
