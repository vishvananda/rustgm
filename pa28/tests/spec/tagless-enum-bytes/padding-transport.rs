#[derive(Copy,Clone)]#[repr(align(8))]enum E{Only(u8)}const N:u8=unsafe{let mut src=E::Only(37);core::ptr::write((&raw mut src)as *mut u8,37);let mut dst=E::Only(0);core::ptr::copy_nonoverlapping(&raw const src,&raw mut dst,1);match dst{E::Only(v)=>v}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
