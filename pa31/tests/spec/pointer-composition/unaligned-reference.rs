#![feature(ptr_metadata,extern_types)]
#![allow(dead_code)]
use core::{ptr,slice};
static DATA:[u8;3]=[7,8,9];
#[repr(C,packed)]struct Packed{pad:u8,x:u16}static P:Packed=Packed{pad:0,x:9};const BAD:&[u16]=unsafe{slice::from_raw_parts(&raw const P.x,1)};
