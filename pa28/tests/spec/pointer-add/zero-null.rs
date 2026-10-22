fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}const P:*const u32=unsafe{(0usize as *const u32).add(0)};fn main(){let _=P;}
