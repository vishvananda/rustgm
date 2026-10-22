const N:u8={let x=0x44332211u32;unsafe{*(&raw const x as *const u8).add(2)}};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn main(){check(N==51);}
