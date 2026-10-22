#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn destination(p:*mut u32,n:&mut u8)->*mut u32{*n+=1;p}fn byte(n:&mut u8)->u8{*n+=1;5}fn count(n:&mut u8)->usize{*n+=1;1}fn main(){let mut x=0u32;let mut n=0u8;unsafe{core::ptr::write_bytes(destination(&raw mut x,&mut n),byte(&mut n),count(&mut n));}check(n==3);check(x==0x05050505);}
