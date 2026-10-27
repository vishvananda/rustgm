#![feature(ptr_metadata,extern_types)]
#![allow(dead_code)]
use core::{ptr,slice};
static DATA:[u8;3]=[7,8,9];
union U{empty:(),byte:u8}
const ANSWER:usize={let x=U{empty:()};let p=&raw const x.byte;ptr::metadata(ptr::slice_from_raw_parts(p,1))};#[unsafe(no_mangle)]pub extern "C" fn answer()->usize{ANSWER}
