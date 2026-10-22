fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}const N:u8={let p=1usize as *const u32;let _=unsafe{p.add(0)};37};fn main(){check(N==37);}
