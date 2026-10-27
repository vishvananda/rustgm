#![feature(ptr_metadata,extern_types)]
#![allow(dead_code)]
use core::{ptr,slice};
static DATA:[u8;3]=[7,8,9];
struct Tail<T:?Sized>{head:u8,tail:T}const ANSWER:usize=ptr::metadata(ptr::from_raw_parts::<Tail<[u8]>>(1usize as *const (),3));#[unsafe(no_mangle)]pub extern "C" fn answer()->usize{ANSWER}
