#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static mut COUNT:u32=0;struct D{n:u8}impl Drop for D{fn drop(&mut self){unsafe{COUNT+=self.n as u32;}}}fn main(){{let mut x=D{n:3};unsafe{core::ptr::write_bytes(&raw mut x,7,1);check(COUNT==0);}}unsafe{check(COUNT==7);}}
