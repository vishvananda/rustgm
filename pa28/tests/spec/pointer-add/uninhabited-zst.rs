fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}enum E{}const P:*const E=unsafe{(1usize as *const E).add(18446744073709551615)};fn main(){let _=P;}
