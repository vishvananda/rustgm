#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
union U{refs:[&'static u32;3],bytes:[u8;24]}const X:[&u32;3]={let source=U{refs:[&A,&B,&C]};let mut target=U{bytes:[0;24]};unsafe{core::ptr::copy_nonoverlapping(&raw const source.bytes[0],&raw mut target.bytes[0],24);target.refs}};static A:u32=11;static B:u32=17;static C:u32=23;
fn main(){check(*X[0]==11 && *X[1]==17 && *X[2]==23);}
