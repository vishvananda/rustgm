const N:bool={let x=1u8;unsafe{*(&raw const x as *const bool)}};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn main(){check(N==true);}
