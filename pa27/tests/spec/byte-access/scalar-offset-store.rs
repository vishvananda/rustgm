const N:u32={let mut x=0x44332211u32;unsafe{(*(&raw mut x as *mut [u8;4]))[2]=0x77;}x};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn main(){check(N==1148658193);}
