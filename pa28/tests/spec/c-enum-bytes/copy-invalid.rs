#[derive(Copy,Clone)]#[repr(C)]enum E{A=0,B=7}union U{e:E,tag:u32}const N:E=unsafe{let src=U{tag:4};let mut dst=U{tag:0};core::ptr::copy_nonoverlapping(&raw const src,&raw mut dst,1);dst.e};fn main(){check(match N{E::A=>true,_=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
