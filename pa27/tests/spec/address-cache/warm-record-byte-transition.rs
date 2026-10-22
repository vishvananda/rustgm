struct S{x:u32,y:u32}const N:u32={let mut s=S{x:37,y:43};let before=s.x;let p=&raw mut s as *mut u8;unsafe{*p=17};let after=s.x;before+after+s.y};
fn main(){check(N==97)}fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
