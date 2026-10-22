const N:u16={let x=[0x44332211u32,0x88776655];unsafe{(*(&raw const x as *const [u16;4]))[3]}};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn main(){check(N==34935);}
