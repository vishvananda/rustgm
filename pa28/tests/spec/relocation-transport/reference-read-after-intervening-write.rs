#![allow(dead_code,unused_mut)]
union U{refs:[&'static u32;3],bytes:[u8;24]}const X:u32={let first=U{refs:[&A,&B,&C]};let second=U{refs:[&B,&C,&A]};let mut target=U{bytes:[0;24]};unsafe{core::ptr::copy_nonoverlapping(&raw const first.bytes[0],&raw mut target.bytes[0],24);}let mut total=0;let mut index=0;while index<2{let values=unsafe{target.refs};total+=*values[0];unsafe{core::ptr::copy_nonoverlapping(&raw const second.bytes[0],&raw mut target.bytes[0],24);}index+=1;}total};static A:u32=11;static B:u32=17;static C:u32=23;
fn main(){let _=X;}
