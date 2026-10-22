const N:u32={let mut x=0x44332211u32;let r=unsafe{&mut *(&raw mut x as *mut u16)};*r=0x7766;x};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn main(){check(N==1144223590);}
