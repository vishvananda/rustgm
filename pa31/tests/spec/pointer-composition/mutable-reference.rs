#![feature(ptr_metadata,extern_types)]
#![allow(dead_code)]
use core::{ptr,slice};
static DATA:[u8;3]=[7,8,9];
const ANSWER:usize={let mut data=[7u8,8u8,9u8];let p=&mut data as *mut [u8;3] as *mut u8;let s=unsafe{slice::from_raw_parts_mut(p,3)};s[1]=91;s[1] as usize};
#[unsafe(no_mangle)]pub extern "C" fn answer()->usize{ANSWER}
