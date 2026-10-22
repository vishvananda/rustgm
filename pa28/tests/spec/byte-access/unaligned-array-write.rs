const N:u8={let mut x=[0u8;4];unsafe{core::ptr::write_unaligned(&raw mut x as *mut u32,0x44332211);}x[2]};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn main(){check(N==51);}
