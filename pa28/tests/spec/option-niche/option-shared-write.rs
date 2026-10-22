#![allow(invalid_reference_casting)]
static S:u32=37;union U{o:Option<&'static u32>,p:&'static u32}const N:u32=unsafe{let r=match(U{p:&S}).o{Some(r)=>r,None=>loop{}};core::ptr::write(r as *const u32 as *mut u32,41);*r};fn main(){let _=N;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
