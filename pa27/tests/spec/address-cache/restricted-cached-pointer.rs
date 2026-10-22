struct S{v:u32}const N:u32=unsafe{let s=S{v:37};let p=&s.v as *const u32 as *mut u32;*p=43;s.v};
fn main(){check(N==43)}fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
