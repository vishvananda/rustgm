#![feature(ptr_metadata,extern_types,core_intrinsics)]
#![allow(dead_code,internal_features)]
use core::{ptr,slice};
static DATA:[u8;3]=[7,8,9];
union U{empty:(),byte:u8}const BAD:u8={let x=U{empty:()};let p=&raw const x.byte;let s=unsafe{slice::from_raw_parts(p,1)};s[0]};
