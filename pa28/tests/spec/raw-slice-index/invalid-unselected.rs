#![allow(dead_code,unused_mut,invalid_reference_casting)]
const X:bool={let mut a=[false;2];let r:&mut [bool]=&mut a;let p=r as *mut [bool];unsafe{core::ptr::write_bytes(&raw mut (*p)[0],2,1);(*p)[1]}};fn main(){let _=X;}
