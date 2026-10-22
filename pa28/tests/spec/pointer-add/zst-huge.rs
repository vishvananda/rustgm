fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}const P:*const ()=unsafe{(1usize as *const ()).add(18446744073709551615)};fn main(){let _=P;}
