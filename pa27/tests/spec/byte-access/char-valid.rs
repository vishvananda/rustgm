const N:char={let x=65u32;unsafe{*(&raw const x as *const char)}};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn main(){check(N=='A');}
