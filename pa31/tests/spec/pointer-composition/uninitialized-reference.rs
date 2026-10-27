#![feature(ptr_metadata,extern_types)]
#![allow(dead_code)]
use core::{ptr,slice};
static DATA:[u8;3]=[7,8,9];
union U{empty:(),byte:u8}
const BAD:()={let x=U{empty:()};let p=&raw const x.byte;let _=unsafe{slice::from_raw_parts(p,1)};};
